use std::{collections::VecDeque, path::{Path, PathBuf}};

use anyhow::Result;
use tokio::{fs, io, select, sync::{mpsc, oneshot}, time};

pub async fn calculate_size(path: &Path) -> u64 {
	let mut total = 0;
	let mut stack = VecDeque::from([path.to_path_buf()]);
	while let Some(path) = stack.pop_front() {
		let Ok(meta) = fs::symlink_metadata(&path).await else {
			continue;
		};

		if !meta.is_dir() {
			total += meta.len();
			continue;
		}

		let Ok(mut it) = fs::read_dir(path).await else {
			continue;
		};

		while let Ok(Some(entry)) = it.next_entry().await {
			let Ok(meta) = entry.metadata().await else {
				continue;
			};

			if meta.is_dir() {
				stack.push_back(entry.path());
			} else {
				total += meta.len();
			}
		}
	}
	total
}

pub fn copy_with_progress(from: &Path, to: &Path) -> mpsc::Receiver<Result<u64, io::Error>> {
	let (tx, rx) = mpsc::channel(1);
	let (tick_tx, mut tick_rx) = oneshot::channel();

	tokio::spawn({
		let (from, to) = (from.to_path_buf(), to.to_path_buf());

		async move {
			_ = match fs::copy(from, to).await {
				Ok(len) => tick_tx.send(Ok(len)),
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
					_ = time::sleep(time::Duration::from_secs(1)) => (),
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
pub fn permissions(mode: u32) -> String {
	use libc::{mode_t, S_IFBLK, S_IFCHR, S_IFDIR, S_IFIFO, S_IFLNK, S_IFMT, S_IFSOCK, S_IRGRP, S_IROTH, S_IRUSR, S_ISGID, S_ISUID, S_ISVTX, S_IWGRP, S_IWOTH, S_IWUSR, S_IXGRP, S_IXOTH, S_IXUSR};

	let mut s = String::with_capacity(10);
	let m = mode as mode_t;

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

// Find the max common root of a list of files
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
