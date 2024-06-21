use std::{borrow::Cow, collections::{HashMap, VecDeque}, ffi::{OsStr, OsString}, fs::Metadata, path::{Path, PathBuf}};

use anyhow::{bail, Result};
use tokio::{fs, io, select, sync::{mpsc, oneshot}, time};

#[inline]
pub async fn must_exists(p: impl AsRef<Path>) -> bool { fs::symlink_metadata(p).await.is_ok() }

#[inline]
pub async fn maybe_exists(p: impl AsRef<Path>) -> bool {
	match fs::symlink_metadata(p).await {
		Ok(_) => true,
		Err(e) => e.kind() != io::ErrorKind::NotFound,
	}
}

#[inline]
pub fn ok_or_not_found(result: io::Result<()>) -> io::Result<()> {
	match result {
		Ok(()) => Ok(()),
		Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
		Err(_) => result,
	}
}

pub async fn symlink_realpath(path: &Path) -> Result<PathBuf> {
	let p = fs::canonicalize(path).await?;
	if p == path {
		return Ok(p);
	}

	let Some(parent) = path.parent() else { bail!("no parent") };
	symlink_realname(path, &mut HashMap::new()).await.map(|n| parent.join(n))
}

#[cfg(unix)]
#[tokio::test]
async fn test_symlink_realpath() {
	fs::remove_dir_all("/tmp/issue-1173").await.ok();
	fs::create_dir_all("/tmp/issue-1173/real-dir").await.unwrap();
	fs::File::create("/tmp/issue-1173/A").await.unwrap();
	fs::File::create("/tmp/issue-1173/b").await.unwrap();
	fs::File::create("/tmp/issue-1173/real-dir/C").await.unwrap();
	fs::symlink("/tmp/issue-1173/b", "/tmp/issue-1173/D").await.unwrap();
	fs::symlink("real-dir", "/tmp/issue-1173/link-dir").await.unwrap();

	async fn check(a: &str, b: &str) {
		let expected = if a == b || cfg!(windows) || cfg!(target_os = "macos") {
			Some(PathBuf::from(b))
		} else {
			None
		};
		assert_eq!(symlink_realpath(Path::new(a)).await.ok(), expected);
	}

	check("/tmp/issue-1173/a", "/tmp/issue-1173/A").await;
	check("/tmp/issue-1173/A", "/tmp/issue-1173/A").await;

	check("/tmp/issue-1173/b", "/tmp/issue-1173/b").await;
	check("/tmp/issue-1173/B", "/tmp/issue-1173/b").await;

	check("/tmp/issue-1173/link-dir/c", "/tmp/issue-1173/link-dir/C").await;
	check("/tmp/issue-1173/link-dir/C", "/tmp/issue-1173/link-dir/C").await;

	check("/tmp/issue-1173/d", "/tmp/issue-1173/D").await;
	check("/tmp/issue-1173/D", "/tmp/issue-1173/D").await;
}

// realpath(3) without resolving symlinks. This is useful for case-insensitive
// filesystems.
//
// Make sure the file of the path exists.
pub async fn symlink_realname<'a>(
	path: &'a Path,
	cached: &'a mut HashMap<PathBuf, HashMap<OsString, OsString>>,
) -> Result<Cow<'a, OsStr>> {
	let Some(name) = path.file_name() else { bail!("no file name") };
	let Some(parent) = path.parent() else { return Ok(name.into()) };

	if !cached.contains_key(parent) {
		let mut map = HashMap::new();
		let mut it = fs::read_dir(parent).await?;
		while let Some(entry) = it.next_entry().await? {
			let n = entry.file_name();
			if n.as_encoded_bytes().iter().all(|&b| b.is_ascii_lowercase()) {
				map.insert(n, OsString::new());
			} else {
				map.insert(n.to_ascii_lowercase(), n);
			}
		}
		cached.insert(parent.to_owned(), map);
	}

	let c = &cached[parent];
	if let Some(s) = c.get(name) {
		return if s.is_empty() { Ok(name.into()) } else { Ok(s.into()) };
	}

	let lowercased = name.to_ascii_lowercase();
	if let Some(s) = c.get(&lowercased) {
		return if s.is_empty() { Ok(lowercased.into()) } else { Ok(s.into()) };
	}

	Ok(name.into())
}

pub async fn calculate_size(path: &Path) -> u64 {
	let mut total = 0;
	let mut stack = VecDeque::from([path.to_path_buf()]);
	while let Some(path) = stack.pop_front() {
		let Ok(meta) = fs::symlink_metadata(&path).await else { continue };
		if !meta.is_dir() {
			total += meta.len();
			continue;
		}

		let Ok(mut it) = fs::read_dir(path).await else { continue };
		while let Ok(Some(entry)) = it.next_entry().await {
			let Ok(meta) = entry.metadata().await else { continue };

			if meta.is_dir() {
				stack.push_back(entry.path());
			} else {
				total += meta.len();
			}
		}
	}
	total
}

pub fn copy_with_progress(
	from: &Path,
	to: &Path,
	meta: &Metadata,
) -> mpsc::Receiver<Result<u64, io::Error>> {
	let (tx, rx) = mpsc::channel(1);
	let (tick_tx, mut tick_rx) = oneshot::channel();

	tokio::spawn({
		let (from, to) = (from.to_owned(), to.to_owned());

		let mut ft = std::fs::FileTimes::new();
		meta.accessed().map(|t| ft = ft.set_accessed(t)).ok();
		meta.modified().map(|t| ft = ft.set_modified(t)).ok();
		#[cfg(target_os = "macos")]
		{
			use std::os::macos::fs::FileTimesExt;
			meta.created().map(|t| ft = ft.set_created(t)).ok();
		}
		#[cfg(windows)]
		{
			use std::os::windows::fs::FileTimesExt;
			meta.created().map(|t| ft = ft.set_created(t)).ok();
		}

		async move {
			_ = match fs::copy(&from, &to).await {
				Ok(len) => {
					_ = tokio::task::spawn_blocking(move || {
						std::fs::File::options().write(true).open(to).and_then(|f| f.set_times(ft)).ok();
					})
					.await;
					tick_tx.send(Ok(len))
				}
				Err(e) => tick_tx.send(Err(e)),
			};
		}
	});

	tokio::spawn({
		let tx = tx.clone();
		let to = to.to_path_buf();

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

				let len = fs::symlink_metadata(&to).await.map(|m| m.len()).unwrap_or(0);
				if len > last {
					tx.send(Ok(len - last)).await.ok();
					last = len;
				}
			}
		}
	});

	rx
}

// Convert a file mode to a string representation
#[cfg(unix)]
#[allow(clippy::collapsible_else_if)]
pub fn permissions(m: libc::mode_t) -> String {
	use libc::{S_IFBLK, S_IFCHR, S_IFDIR, S_IFIFO, S_IFLNK, S_IFMT, S_IFSOCK, S_IRGRP, S_IROTH, S_IRUSR, S_ISGID, S_ISUID, S_ISVTX, S_IWGRP, S_IWOTH, S_IWUSR, S_IXGRP, S_IXOTH, S_IXUSR};
	let mut s = String::with_capacity(10);

	// File type
	s.push(match m & S_IFMT {
		S_IFBLK => 'b',
		S_IFCHR => 'c',
		S_IFDIR => 'd',
		S_IFIFO => 'p',
		S_IFLNK => 'l',
		S_IFSOCK => 's',
		_ => '-',
	});

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

// Find the max common root in a list of files
// e.g. /a/b/c, /a/b/d       -> /a/b
//      /aa/bb/cc, /aa/dd/ee -> /aa
pub fn max_common_root(files: &[impl AsRef<Path>]) -> PathBuf {
	if files.is_empty() {
		return PathBuf::new();
	}

	let mut it = files.iter().map(|p| p.as_ref().parent().unwrap_or(Path::new("")).components());
	let mut root = it.next().unwrap().collect::<PathBuf>();
	for components in it {
		let mut new_root = PathBuf::new();
		for (a, b) in root.components().zip(components) {
			if a != b {
				break;
			}
			new_root.push(a);
		}
		root = new_root;
	}
	root
}

#[cfg(unix)]
#[test]
fn test_max_common_root() {
	assert_eq!(max_common_root(&[] as &[PathBuf]).as_os_str(), "");
	assert_eq!(max_common_root(&["".into()] as &[PathBuf]).as_os_str(), "");
	assert_eq!(max_common_root(&["a".into()] as &[PathBuf]).as_os_str(), "");
	assert_eq!(max_common_root(&["/a".into()] as &[PathBuf]).as_os_str(), "/");
	assert_eq!(max_common_root(&["/a/b".into()] as &[PathBuf]).as_os_str(), "/a");
	assert_eq!(
		max_common_root(&["/a/b/c".into(), "/a/b/d".into()] as &[PathBuf]).as_os_str(),
		"/a/b"
	);
	assert_eq!(
		max_common_root(&["/aa/bb/cc".into(), "/aa/dd/ee".into()] as &[PathBuf]).as_os_str(),
		"/aa"
	);
	assert_eq!(
		max_common_root(
			&["/aa/bb/cc".into(), "/aa/bb/cc/dd/ee".into(), "/aa/bb/cc/ff".into()] as &[PathBuf]
		)
		.as_os_str(),
		"/aa/bb"
	);
}
