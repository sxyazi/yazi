use std::{collections::VecDeque, io, time::{Duration, Instant}};

use yazi_fs::provider::{DirReader, FileHolder};
use yazi_shared::{Either, url::{AsUrl, UrlBuf}};

use super::ReadDir;

pub enum SizeCalculator {
	File(Option<u64>),
	Dir(VecDeque<Either<UrlBuf, ReadDir>>),
}

impl SizeCalculator {
	pub async fn new<U>(url: U) -> io::Result<Self>
	where
		U: AsUrl,
	{
		let url = url.as_url();
		let cha = super::symlink_metadata(url).await?;
		Ok(if cha.is_dir() {
			Self::Dir(VecDeque::from([Either::Left(url.to_owned())]))
		} else {
			Self::File(Some(cha.len))
		})
	}

	pub async fn total<U>(url: U) -> io::Result<u64>
	where
		U: AsUrl,
	{
		let mut it = Self::new(url).await?;
		let mut total = 0;
		while let Some(n) = it.next().await? {
			total += n;
		}
		Ok(total)
	}

	pub async fn next(&mut self) -> io::Result<Option<u64>> {
		Ok(match self {
			Self::File(size) => size.take(),
			Self::Dir(buf) => Self::next_chunk(buf).await,
		})
	}

	async fn next_chunk(buf: &mut VecDeque<Either<UrlBuf, ReadDir>>) -> Option<u64> {
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

		while i < 2000 && now.elapsed() < Duration::from_millis(100) {
			i += 1;
			let front = buf.front_mut()?;

			if let Either::Left(p) = front {
				*front = match super::read_dir(p).await {
					Ok(it) => Either::Right(it),
					Err(_) => pop_and_continue!(),
				};
			}

			let Ok(Some(ent)) = front.right_mut()?.next().await else {
				pop_and_continue!();
			};

			let Ok(ft) = ent.file_type().await else { continue };
			if ft.is_dir() {
				buf.push_back(Either::Left(ent.url()));
			} else if let Ok(cha) = ent.metadata().await {
				size += cha.len;
			}
		}
		Some(size)
	}
}
