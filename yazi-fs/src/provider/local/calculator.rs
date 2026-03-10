use std::{collections::VecDeque, future::poll_fn, io, mem, path::{Path, PathBuf}, pin::Pin, task::{Poll, ready}, time::{Duration, Instant}};

use either::Either;
use tokio::task::JoinHandle;

use crate::cha::Cha;

type Task = Either<PathBuf, std::fs::ReadDir>;

pub enum SizeCalculator {
	Idle((VecDeque<Task>, Option<u64>), Cha),
	Pending(JoinHandle<(VecDeque<Task>, Option<u64>)>, Cha),
}

impl SizeCalculator {
	pub async fn new(path: &Path) -> io::Result<Self> {
		let p = path.to_owned();
		tokio::task::spawn_blocking(move || {
			let cha = Cha::new(p.file_name().unwrap_or_default(), std::fs::symlink_metadata(&p)?);
			if !cha.is_dir() {
				return Ok(Self::Idle((VecDeque::new(), Some(cha.len)), cha));
			}

			let mut buf = VecDeque::from([Either::Right(std::fs::read_dir(&p)?)]);
			let size = Self::next_chunk(&mut buf);
			Ok(Self::Idle((buf, size), cha))
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
					Self::Idle((buf, size), cha) => {
						if let Some(s) = size.take() {
							return Poll::Ready(Ok(Some(s)));
						} else if buf.is_empty() {
							return Poll::Ready(Ok(None));
						}

						let mut buf = mem::take(buf);
						*self = Self::Pending(
							tokio::task::spawn_blocking(move || {
								let size = Self::next_chunk(&mut buf);
								(buf, size)
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

	fn next_chunk(buf: &mut VecDeque<Either<PathBuf, std::fs::ReadDir>>) -> Option<u64> {
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
				buf.push_back(Either::Left(ent.path()));
			} else if let Ok(meta) = ent.metadata() {
				size += meta.len();
			}
		}
		Some(size)
	}
}
