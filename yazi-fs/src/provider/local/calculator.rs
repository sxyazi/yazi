use std::{collections::VecDeque, future::poll_fn, io, mem, path::{Path, PathBuf}, pin::Pin, task::{Poll, ready}, time::{Duration, Instant}};

use either::Either;
use tokio::task::JoinHandle;

#[cfg(target_os = "linux")]
use crate::mounts::Partition;
use crate::cha::Cha;

type Task = Either<PathBuf, std::fs::ReadDir>;
#[cfg(target_os = "linux")]
type SystemicMounts = std::collections::HashSet<PathBuf>;
#[cfg(not(target_os = "linux"))]
type SystemicMounts = ();

pub enum SizeCalculator {
	Idle((VecDeque<Task>, Option<u64>, SystemicMounts), Cha),
	Pending(JoinHandle<(VecDeque<Task>, Option<u64>, SystemicMounts)>, Cha),
}

impl SizeCalculator {
	pub async fn new(path: &Path) -> io::Result<Self> {
		let p = path.to_owned();
		tokio::task::spawn_blocking(move || {
			let cha = Cha::new(p.file_name().unwrap_or_default(), std::fs::symlink_metadata(&p)?);
			if !cha.is_dir() {
				return Ok(Self::Idle((VecDeque::new(), Some(cha.len), systemic_mounts(&p)), cha));
			}

			let systemic = systemic_mounts(&p);
			if is_systemic_mount(&p, &systemic) {
				return Ok(Self::Idle((VecDeque::new(), Some(0), systemic), cha));
			}

			let mut buf = VecDeque::from([Either::Right(std::fs::read_dir(&p)?)]);
			let size = Self::next_chunk(&mut buf, &systemic);
			Ok(Self::Idle((buf, size, systemic), cha))
		})
		.await?
	}

	pub fn cha(&self) -> Cha {
		match *self {
			Self::Idle(_, cha) | Self::Pending(_, cha) => cha,
		}
	}

	pub async fn total(path: &Path) -> io::Result<u64> {
		let mut it = Self::new(path).await?;
		let mut total = 0;
		while let Some(n) = it.next().await? {
			total += n;
		}
		Ok(total)
	}

	pub async fn next(&mut self) -> io::Result<Option<u64>> {
		poll_fn(|cx| {
			loop {
				match self {
					Self::Idle((buf, size, systemic), cha) => {
						if let Some(s) = size.take() {
							return Poll::Ready(Ok(Some(s)));
						} else if buf.is_empty() {
							return Poll::Ready(Ok(None));
						}

						let mut buf = mem::take(buf);
						let systemic = mem::take(systemic);
						*self = Self::Pending(
							tokio::task::spawn_blocking(move || {
								let size = Self::next_chunk(&mut buf, &systemic);
								(buf, size, systemic)
							}),
							*cha,
						);
					}
					Self::Pending(handle, cha) => {
						*self = Self::Idle(ready!(Pin::new(handle).poll(cx))?, *cha);
					}
				}
			}
		})
		.await
	}

	fn next_chunk(
		buf: &mut VecDeque<Either<PathBuf, std::fs::ReadDir>>,
		systemic: &SystemicMounts,
	) -> Option<u64> {
		let (mut i, mut size, now) = (0, 0, Instant::now());
		macro_rules! pop_and_continue {
			() => {{
				buf.pop_front();
				if buf.is_empty() {
					return Some(size);
				}
				continue;
			}};
		}

		while i < 5000 && now.elapsed() < Duration::from_millis(50) {
			i += 1;
			let front = buf.front_mut()?;

			if let Either::Left(p) = front {
				*front = match std::fs::read_dir(p) {
					Ok(it) => Either::Right(it),
					Err(_) => pop_and_continue!(),
				};
			}

			let Some(next) = front.as_mut().right()?.next() else {
				pop_and_continue!();
			};

			let Ok(ent) = next else { continue };
			let Ok(ft) = ent.file_type() else { continue };
			if ft.is_dir() {
				let path = ent.path();
				if !is_systemic_mount(&path, systemic) {
					buf.push_back(Either::Left(path));
				}
			} else if let Ok(meta) = ent.metadata() {
				size += meta.len();
			}
		}
		Some(size)
	}
}

#[cfg(target_os = "linux")]
fn systemic_mounts(root: &Path) -> SystemicMounts {
	systemic_mounts_from(root, &std::fs::read_to_string("/proc/mounts").unwrap_or_default())
}

#[cfg(not(target_os = "linux"))]
fn systemic_mounts(_: &Path) -> SystemicMounts {}

#[cfg(target_os = "linux")]
fn is_systemic_mount(path: &Path, systemic: &SystemicMounts) -> bool { systemic.contains(path) }

#[cfg(not(target_os = "linux"))]
fn is_systemic_mount(_: &Path, _: &SystemicMounts) -> bool { false }

#[cfg(target_os = "linux")]
fn systemic_mounts_from(root: &Path, mounts: &str) -> SystemicMounts {
	mounts
		.lines()
		.filter_map(|line| {
			let mut it = line.split_whitespace();
			let _src = it.next()?;
			let dist = unmangle_octal(it.next()?);
			let fstype = unmangle_octal(it.next()?);
			let dist = PathBuf::from(dist.as_ref());
			let systemic = Partition { fstype: Some(fstype.into_owned().into()), ..Default::default() }
				.systemic();
			let descendant = dist == root || dist.starts_with(root);

			(systemic && descendant).then_some(dist)
		})
		.collect()
}

#[cfg(target_os = "linux")]
fn unmangle_octal(s: &str) -> std::borrow::Cow<'_, str> {
	use yazi_shared::replace_cow;

	let mut s = std::borrow::Cow::Borrowed(s);
	// `/proc/mounts` escapes tabs, newlines, spaces, `#`, and `\` using octal sequences.
	for (a, b) in
		[(r"\011", "\t"), (r"\012", "\n"), (r"\040", " "), (r"\043", "#"), (r"\134", r"\")]
	{
		s = replace_cow(s, a, b);
	}
	s
}

#[cfg(test)]
mod tests {
	use std::{path::Path, sync::OnceLock};

	use super::SizeCalculator;

	#[cfg(target_os = "linux")]
	use super::systemic_mounts_from;

	fn init() {
		static INIT: OnceLock<()> = OnceLock::new();
		INIT.get_or_init(crate::init);
	}

	#[cfg(target_os = "linux")]
	#[test]
	fn systemic_mounts_only_include_systemic_descendants() {
		let mounts = systemic_mounts_from(
			Path::new("/"),
			"rootfs / ext4 rw 0 0\nproc /proc proc rw 0 0\nsysfs /sys sysfs rw 0 0\ntmpfs /tmp tmpfs rw 0 0\n/dev/sda1 /home ext4 rw 0 0\n",
		);

		assert!(mounts.contains(Path::new("/proc")));
		assert!(mounts.contains(Path::new("/sys")));
		assert!(mounts.contains(Path::new("/tmp")));
		assert!(!mounts.contains(Path::new("/home")));
	}

	#[cfg(target_os = "linux")]
	#[tokio::test]
	async fn proc_size_ignores_systemic_pseudo_files() {
		init();

		let mut it = SizeCalculator::new(Path::new("/proc")).await.unwrap();
		assert_eq!(it.next().await.unwrap(), Some(0));
		assert_eq!(it.next().await.unwrap(), None);
	}
}
