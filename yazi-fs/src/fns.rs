// FIXME: VFS

use std::{borrow::Cow, collections::{HashMap, HashSet}, ffi::{OsStr, OsString}, path::{Path, PathBuf}};

use anyhow::{Result, bail};
use tokio::{fs, io, select, sync::{mpsc, oneshot}, time};
use yazi_shared::url::{Component, Url, UrlBuf};

use crate::{cha::Cha, provider};

#[inline]
pub async fn maybe_exists<'a>(url: impl Into<Url<'a>>) -> bool {
	match provider::symlink_metadata(url).await {
		Ok(_) => true,
		Err(e) => e.kind() != io::ErrorKind::NotFound,
	}
}

#[inline]
pub async fn must_be_dir<'a>(url: impl Into<Url<'a>>) -> bool {
	provider::metadata(url).await.is_ok_and(|m| m.is_dir())
}

#[inline]
pub fn ok_or_not_found<T: Default>(result: io::Result<T>) -> io::Result<T> {
	match result {
		Ok(t) => Ok(t),
		Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(T::default()),
		Err(_) => result,
	}
}

#[inline]
pub async fn paths_to_same_file(a: impl AsRef<Path>, b: impl AsRef<Path>) -> bool {
	_paths_to_same_file(a.as_ref(), b.as_ref()).await.unwrap_or(false)
}

#[cfg(unix)]
async fn _paths_to_same_file(a: &Path, b: &Path) -> io::Result<bool> {
	use std::os::unix::fs::MetadataExt;

	let (a_, b_) = (fs::symlink_metadata(a).await?, fs::symlink_metadata(b).await?);
	Ok(
		a_.ino() == b_.ino()
			&& a_.dev() == b_.dev()
			&& fs::canonicalize(a).await? == fs::canonicalize(b).await?,
	)
}

#[cfg(windows)]
async fn _paths_to_same_file(a: &Path, b: &Path) -> std::io::Result<bool> {
	use std::os::windows::{ffi::OsStringExt, io::AsRawHandle};

	use windows_sys::Win32::{Foundation::{HANDLE, MAX_PATH}, Storage::FileSystem::{FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT, GetFinalPathNameByHandleW, VOLUME_NAME_DOS}};

	async fn final_name(p: &Path) -> std::io::Result<PathBuf> {
		let file = fs::OpenOptions::new()
			.access_mode(0)
			.custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
			.open(p)
			.await?;

		tokio::task::spawn_blocking(move || {
			let mut buf = [0u16; MAX_PATH as usize];
			let len = unsafe {
				GetFinalPathNameByHandleW(
					file.as_raw_handle() as HANDLE,
					buf.as_mut_ptr(),
					buf.len() as u32,
					VOLUME_NAME_DOS,
				)
			};

			if len == 0 {
				Err(std::io::Error::last_os_error())
			} else {
				Ok(PathBuf::from(OsString::from_wide(&buf[0..len as usize])))
			}
		})
		.await?
	}

	Ok(final_name(a).await? == final_name(b).await?)
}

pub async fn realname(u: &UrlBuf) -> Option<OsString> {
	let name = u.file_name()?;
	if *u == provider::canonicalize(u).await.ok()? {
		return None;
	}

	realname_unchecked(u, &mut HashMap::new())
		.await
		.ok()
		.filter(|s| s != name)
		.map(|s| s.into_owned())
}

#[cfg(unix)]
#[tokio::test]
async fn test_realname_unchecked() -> Result<()> {
	use crate::provider::local::Local;

	Local::remove_dir_all("/tmp/issue-1173").await.ok();
	Local::create_dir_all("/tmp/issue-1173/real-dir").await?;
	Local::create("/tmp/issue-1173/A").await?;
	Local::create("/tmp/issue-1173/b").await?;
	Local::create("/tmp/issue-1173/real-dir/C").await?;
	Local::symlink_file("/tmp/issue-1173/b", "/tmp/issue-1173/D").await?;
	Local::symlink_dir("real-dir", "/tmp/issue-1173/link-dir").await?;

	let c = &mut HashMap::new();
	async fn check(a: &str, b: &str, c: &mut HashMap<PathBuf, HashSet<OsString>>) {
		assert_eq!(realname_unchecked(Path::new(a), c).await.ok(), Some(OsStr::new(b).into()));
	}

	check("/tmp/issue-1173/a", "A", c).await;
	check("/tmp/issue-1173/A", "A", c).await;

	check("/tmp/issue-1173/b", "b", c).await;
	check("/tmp/issue-1173/B", "b", c).await;

	check("/tmp/issue-1173/link-dir/c", "C", c).await;
	check("/tmp/issue-1173/link-dir/C", "C", c).await;

	check("/tmp/issue-1173/d", "D", c).await;
	check("/tmp/issue-1173/D", "D", c).await;
	Ok(())
}

// realpath(3) without resolving symlinks. This is useful for case-insensitive
// filesystems.
//
// Make sure the file of the path exists.
pub async fn realname_unchecked<'a>(
	path: &'a Path,
	cached: &'a mut HashMap<PathBuf, HashSet<OsString>>,
) -> Result<Cow<'a, OsStr>> {
	let Some(name) = path.file_name() else { bail!("no filename") };
	let Some(parent) = path.parent() else { return Ok(Cow::Borrowed(name)) };

	if !cached.contains_key(parent) {
		let mut set = HashSet::new();
		let mut it = fs::read_dir(parent).await?;
		while let Some(entry) = it.next_entry().await? {
			set.insert(entry.file_name());
		}
		cached.insert(parent.to_owned(), set);
	}

	let c = &cached[parent];
	if c.contains(name) {
		Ok(Cow::Borrowed(name))
	} else if let Some(n) = c.iter().find(|&n| n.eq_ignore_ascii_case(name)) {
		Ok(Cow::Borrowed(n))
	} else {
		bail!("no such file")
	}
}

pub fn copy_with_progress(
	from: &UrlBuf,
	to: &UrlBuf,
	cha: Cha,
) -> mpsc::Receiver<Result<u64, io::Error>> {
	let (tx, rx) = mpsc::channel(1);
	let (tick_tx, mut tick_rx) = oneshot::channel();

	tokio::spawn({
		let (from, to) = (from.clone(), to.clone());
		async move {
			tick_tx.send(_copy_with_progress(from, to, cha).await).ok();
		}
	});

	tokio::spawn({
		let (tx, to) = (tx.clone(), to.clone());
		async move {
			let mut last = 0;
			let mut exit = None;
			loop {
				select! {
					res = &mut tick_rx => exit = Some(res.unwrap()),
					_ = tx.closed() => break,
					_ = time::sleep(time::Duration::from_secs(3)) => (),
				}

				match exit {
					Some(Ok(len)) => {
						if len > last {
							tx.send(Ok(len - last)).await.ok();
						}
						tx.send(Ok(0)).await.ok();
						break;
					}
					Some(Err(e)) => {
						tx.send(Err(e)).await.ok();
						break;
					}
					None => {}
				}

				let len = provider::symlink_metadata(&to).await.map(|m| m.len()).unwrap_or(0);
				if len > last {
					tx.send(Ok(len - last)).await.ok();
					last = len;
				}
			}
		}
	});

	rx
}

async fn _copy_with_progress(from: UrlBuf, to: UrlBuf, cha: Cha) -> io::Result<u64> {
	let mut ft = std::fs::FileTimes::new();
	cha.atime.map(|t| ft = ft.set_accessed(t));
	cha.mtime.map(|t| ft = ft.set_modified(t));
	#[cfg(target_os = "macos")]
	{
		use std::os::macos::fs::FileTimesExt;
		cha.btime.map(|t| ft = ft.set_created(t));
	}
	#[cfg(windows)]
	{
		use std::os::windows::fs::FileTimesExt;
		cha.btime.map(|t| ft = ft.set_created(t));
	}

	#[cfg(any(target_os = "linux", target_os = "android"))]
	{
		use std::os::{fd::AsRawFd, unix::fs::OpenOptionsExt};

		tokio::task::spawn_blocking(move || {
			let mut reader = std::fs::File::open(from)?;
			let mut writer = std::fs::OpenOptions::new()
				.mode(cha.mode as u32)  // Do not remove `as u32`, https://github.com/termux/termux-packages/pull/22481
				.write(true)
				.create(true)
				.truncate(true)
				.open(to)?;

			let written = std::io::copy(&mut reader, &mut writer)?;
			unsafe { libc::fchmod(writer.as_raw_fd(), cha.mode) };
			writer.set_times(ft).ok();

			Ok(written)
		})
		.await?
	}

	#[cfg(not(any(target_os = "linux", target_os = "android")))]
	{
		tokio::task::spawn_blocking(move || {
			let written = std::fs::copy(from, &to)?;
			std::fs::File::options().write(true).open(to).and_then(|f| f.set_times(ft)).ok();
			Ok(written)
		})
		.await?
	}
}

pub async fn remove_dir_clean(dir: &UrlBuf) {
	let Ok(mut it) = provider::read_dir(dir).await else { return };

	while let Ok(Some(entry)) = it.next_entry().await {
		if entry.file_type().await.is_ok_and(|t| t.is_dir()) {
			let url = entry.url();
			Box::pin(remove_dir_clean(&url)).await;
			provider::remove_dir(&url).await.ok();
		}
	}

	provider::remove_dir(dir).await.ok();
}

// Convert a file mode to a string representation
#[cfg(unix)]
#[allow(clippy::collapsible_else_if)]
pub fn permissions(m: libc::mode_t, dummy: bool) -> String {
	use libc::{S_IFBLK, S_IFCHR, S_IFDIR, S_IFIFO, S_IFLNK, S_IFMT, S_IFSOCK, S_IRGRP, S_IROTH, S_IRUSR, S_ISGID, S_ISUID, S_ISVTX, S_IWGRP, S_IWOTH, S_IWUSR, S_IXGRP, S_IXOTH, S_IXUSR};
	let mut s = String::with_capacity(10);

	// Filetype
	s.push(match m & S_IFMT {
		S_IFBLK => 'b',
		S_IFCHR => 'c',
		S_IFDIR => 'd',
		S_IFIFO => 'p',
		S_IFLNK => 'l',
		S_IFSOCK => 's',
		_ => '-',
	});

	if dummy {
		s.push_str("?????????");
		return s;
	}

	// Owner
	s.push(if m & S_IRUSR != 0 { 'r' } else { '-' });
	s.push(if m & S_IWUSR != 0 { 'w' } else { '-' });
	s.push(if m & S_IXUSR != 0 {
		if m & S_ISUID != 0 { 's' } else { 'x' }
	} else {
		if m & S_ISUID != 0 { 'S' } else { '-' }
	});

	// Group
	s.push(if m & S_IRGRP != 0 { 'r' } else { '-' });
	s.push(if m & S_IWGRP != 0 { 'w' } else { '-' });
	s.push(if m & S_IXGRP != 0 {
		if m & S_ISGID != 0 { 's' } else { 'x' }
	} else {
		if m & S_ISGID != 0 { 'S' } else { '-' }
	});

	// Other
	s.push(if m & S_IROTH != 0 { 'r' } else { '-' });
	s.push(if m & S_IWOTH != 0 { 'w' } else { '-' });
	s.push(if m & S_IXOTH != 0 {
		if m & S_ISVTX != 0 { 't' } else { 'x' }
	} else {
		if m & S_ISVTX != 0 { 'T' } else { '-' }
	});

	s
}

// Find the max common root in a list of urls
// e.g. /a/b/c, /a/b/d       -> /a/b
//      /aa/bb/cc, /aa/dd/ee -> /aa
pub fn max_common_root(urls: &[UrlBuf]) -> usize {
	if urls.is_empty() {
		return 0;
	} else if urls.len() == 1 {
		return urls[0].components().count() - 1;
	}

	let mut it = urls.iter().map(|u| u.parent_url());
	let Some(first) = it.next().unwrap() else {
		return 0; // The first URL has no parent
	};

	let mut common = first.components().count();
	for parent in it {
		let Some(parent) = parent else {
			return 0; // One of the URLs has no parent
		};

		common = first
			.components()
			.zip(parent.components())
			.take_while(|(a, b)| match (a, b) {
				(Component::Scheme(a), Component::Scheme(b)) => a.covariant(b),
				(a, b) => a == b,
			})
			.count()
			.min(common);

		if common == 0 {
			break; // No common root found
		}
	}

	common
}

#[cfg(unix)]
#[test]
fn test_max_common_root() {
	fn assert(input: &[&str], expected: &str) {
		use std::str::FromStr;
		let urls: Vec<_> = input.iter().copied().map(UrlBuf::from_str).collect::<Result<_>>().unwrap();

		let mut comp = urls[0].components();
		for _ in 0..comp.clone().count() - max_common_root(&urls) {
			comp.next_back();
		}
		assert_eq!(comp.os_str(), OsStr::new(expected));
	}

	assert_eq!(max_common_root(&[]), 0);
	assert(&[""], "");
	assert(&["a"], "");

	assert(&["/a"], "/");
	assert(&["/a/b"], "/a");
	assert(&["/a/b/c", "/a/b/d"], "/a/b");
	assert(&["/aa/bb/cc", "/aa/dd/ee"], "/aa");
	assert(&["/aa/bb/cc", "/aa/bb/cc/dd/ee", "/aa/bb/cc/ff"], "/aa/bb");
}
