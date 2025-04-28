use std::{collections::VecDeque, fs::ReadDir, future::poll_fn, mem, path::{Path, PathBuf}, pin::Pin, task::{Poll, ready}, time::{Duration, Instant}};

use tokio::task::JoinHandle;
use yazi_shared::Either;

type Task = Either<PathBuf, ReadDir>;

pub enum SizeCalculator {
	Idle((VecDeque<Task>, Option<u64>)),
	Pending(JoinHandle<(VecDeque<Task>, Option<u64>)>),
}

impl SizeCalculator {
	pub async fn new(path: impl AsRef<Path>) -> std::io::Result<Self> {
		let p = path.as_ref().to_owned();
		tokio::task::spawn_blocking(move || {
			let meta = std::fs::symlink_metadata(&p)?;
			if !meta.is_dir() {
				return Ok(Self::Idle((VecDeque::new(), Some(meta.len()))));
			}

			let mut buf = VecDeque::from([Either::Right(std::fs::read_dir(p)?)]);
			let size = Self::next_chunk(&mut buf);
			Ok(Self::Idle((buf, size)))
		})
		.await?
	}

	pub async fn total(path: impl AsRef<Path>) -> std::io::Result<u64> {
		let mut it = Self::new(path).await?;
		let mut total = 0;
		while let Some(n) = it.next().await? {
			total += n;
		}
		Ok(total)
	}

	pub async fn next(&mut self) -> std::io::Result<Option<u64>> {
		poll_fn(|cx| {
			loop {
				match self {
					Self::Idle((buf, size)) => {
						if let Some(s) = size.take() {
							return Poll::Ready(Ok(Some(s)));
						} else if buf.is_empty() {
							return Poll::Ready(Ok(None));
						}

						let mut buf = mem::take(buf);
						*self = Self::Pending(tokio::task::spawn_blocking(move || {
							let size = Self::next_chunk(&mut buf);
							(buf, size)
						}));
					}
					Self::Pending(handle) => {
						*self = Self::Idle(ready!(Pin::new(handle).poll(cx))?);
					}
				}
			}
		})
		.await
	}

	fn next_chunk(buf: &mut VecDeque<Either<PathBuf, ReadDir>>) -> Option<u64> {
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

			let Some(next) = front.right_mut()?.next() else {
				pop_and_continue!();
			};

			let Ok(ent) = next else { continue };
			let Ok(ft) = ent.file_type() else { continue };
			if ft.is_dir() {
				buf.push_back(Either::Left(ent.path()));
			} else if let Ok(meta) = ent.metadata() {
				size += meta.len();
			}
		}
		Some(size)
	}
}
