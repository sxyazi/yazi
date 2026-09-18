use std::{collections::VecDeque, fmt::Debug};

use yazi_fs::{FsUrl, engine::{DirReader, FileHolder}, path::skip_url, stat::Stat};
use yazi_shared::{strand::StrandLike, url::{AsUrl, Url, UrlBuf, UrlLike}};
use yazi_vfs::engine::{self};

use crate::{ctx, file::{FileInCopy, FileInDelete, FileInDownload, FileInHardlink, FileInMove, FileInUpload}};

pub(super) trait Traverse {
	fn stat(&mut self) -> &mut Option<Stat>;

	fn follow(&self) -> bool;

	fn from(&self) -> Url<'_>;

	async fn init(&mut self) -> anyhow::Result<Stat> {
		if self.stat().is_none() {
			*self.stat() = Some(super::File::stat(self.from(), self.follow(), None).await?)
		}
		Ok(self.stat().unwrap())
	}

	fn spawn(&self, from: UrlBuf, to: Option<UrlBuf>, stat: Stat) -> Self;

	fn to(&self) -> Option<Url<'_>>;
}

impl Traverse for FileInCopy {
	fn stat(&mut self) -> &mut Option<Stat> { &mut self.stat }

	fn follow(&self) -> bool { self.follow }

	fn from(&self) -> Url<'_> { self.from.as_url() }

	fn spawn(&self, from: UrlBuf, to: Option<UrlBuf>, stat: Stat) -> Self {
		Self {
			id: self.id,
			from,
			to: to.unwrap(),
			force: self.force,
			stat: Some(stat),
			follow: self.follow,
			retry: self.retry,
		}
	}

	fn to(&self) -> Option<Url<'_>> { Some(self.to.as_url()) }
}

impl Traverse for FileInMove {
	fn stat(&mut self) -> &mut Option<Stat> { &mut self.stat }

	fn follow(&self) -> bool { self.follow }

	fn from(&self) -> Url<'_> { self.from.as_url() }

	fn spawn(&self, from: UrlBuf, to: Option<UrlBuf>, stat: Stat) -> Self {
		Self {
			id: self.id,
			from,
			to: to.unwrap(),
			force: self.force,
			stat: Some(stat),
			follow: self.follow,
			retry: self.retry,
			drop: self.drop.clone(),
		}
	}

	fn to(&self) -> Option<Url<'_>> { Some(self.to.as_url()) }
}

impl Traverse for FileInHardlink {
	fn stat(&mut self) -> &mut Option<Stat> { &mut self.stat }

	fn follow(&self) -> bool { self.follow }

	fn from(&self) -> Url<'_> { self.from.as_url() }

	fn spawn(&self, from: UrlBuf, to: Option<UrlBuf>, stat: Stat) -> Self {
		Self {
			id: self.id,
			from,
			to: to.unwrap(),
			force: self.force,
			stat: Some(stat),
			follow: self.follow,
		}
	}

	fn to(&self) -> Option<Url<'_>> { Some(self.to.as_url()) }
}

impl Traverse for FileInDelete {
	fn stat(&mut self) -> &mut Option<Stat> { &mut self.stat }

	fn follow(&self) -> bool { false }

	fn from(&self) -> Url<'_> { self.target.as_url() }

	fn spawn(&self, from: UrlBuf, _to: Option<UrlBuf>, stat: Stat) -> Self {
		Self { id: self.id, target: from, stat: Some(stat) }
	}

	fn to(&self) -> Option<Url<'_>> { None }
}

impl Traverse for FileInDownload {
	fn stat(&mut self) -> &mut Option<Stat> { &mut self.stat }

	fn follow(&self) -> bool { true }

	fn from(&self) -> Url<'_> { self.target.as_url() }

	fn spawn(&self, from: UrlBuf, _to: Option<UrlBuf>, stat: Stat) -> Self {
		Self { id: self.id, target: from, stat: Some(stat), retry: self.retry }
	}

	fn to(&self) -> Option<Url<'_>> { None }
}

impl Traverse for FileInUpload {
	fn stat(&mut self) -> &mut Option<Stat> { &mut self.stat }

	fn follow(&self) -> bool { true }

	fn from(&self) -> Url<'_> { self.target.as_url() }

	async fn init(&mut self) -> anyhow::Result<Stat> {
		if self.stat.is_none() {
			self.stat = Some(super::File::stat(self.from(), self.follow(), None).await?)
		}
		if self.cache.is_none() {
			self.cache = self.target.cache_entry();
		}
		Ok(self.stat.unwrap())
	}

	fn spawn(&self, from: UrlBuf, _to: Option<UrlBuf>, stat: Stat) -> Self {
		Self { id: self.id, stat: Some(stat), cache: from.cache_entry(), target: from }
	}

	fn to(&self) -> Option<Url<'_>> { None }
}

#[allow(private_bounds)]
pub(super) async fn traverse<O, I, D, FC, FR, E>(
	mut task: I,
	on_dir: D,
	mut on_file: FC,
	on_error: E,
) -> Result<(), O>
where
	O: Debug + From<anyhow::Error>,
	I: Debug + Traverse,
	D: AsyncFn(Url) -> Result<(), O>,
	FC: FnMut(I, Stat) -> FR,
	FR: Future<Output = Result<(), O>>,
	E: Fn(String),
{
	let stat = ctx!(task, task.init().await)?;
	let follow_symlink = stat.is_link() && task.follow();
	if !stat.is_dir() || (!follow_symlink && stat.is_indirect()) {
		return on_file(task, stat).await;
	}

	let root = task.to();
	let skip = task.from().components().count();
	let mut dirs = VecDeque::from([task.from().to_owned()]);

	macro_rules! err {
		($result:expr, $($args:tt)*) => {
			match $result {
				Ok(v) => v,
				Err(e) => {
					on_error(format!("{}: {e:?}", format_args!($($args)*)));
					continue;
				}
			}
		};
	}

	while let Some(src) = dirs.pop_front() {
		let mut it = err!(engine::read_dir(&src).await, "Cannot read directory {src}");

		let dest = if let Some(root) = root {
			let s = skip_url(&src, skip);
			err!(root.try_join(&s), "Cannot join {root} with {}", s.display())
		} else {
			src
		};

		() = err!(on_dir(dest.as_url()).await, "Cannot process directory {dest}");

		while let Ok(Some(dent)) = it.next().await {
			let from = dent.url();
			let stat = err!(
				super::File::stat(&from, task.follow(), Some(dent)).await,
				"Cannot get metadata for {from}"
			);

			let follow_symlink = stat.is_link() && task.follow();
			if stat.is_dir() && (follow_symlink || !stat.is_indirect()) {
				dirs.push_back(from);
				continue;
			}

			let to = if root.is_some() {
				let name = from.name().unwrap();
				Some(err!(dest.try_join(name), "Cannot join {dest} with {}", name.display()))
			} else {
				None
			};

			err!(on_file(task.spawn(from, to, stat), stat).await, "Cannot process file");
		}
	}

	Ok(())
}
