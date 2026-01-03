use std::{collections::VecDeque, fmt::Debug};

use yazi_fs::{FsUrl, cha::Cha, path::skip_url, provider::{DirReader, FileHolder}};
use yazi_shared::{strand::StrandLike, url::{AsUrl, Url, UrlBuf, UrlLike}};
use yazi_vfs::provider::{self};

use crate::{ctx, file::{FileInCopy, FileInCut, FileInDelete, FileInDownload, FileInHardlink, FileInUpload}};

pub(super) trait Traverse {
	fn cha(&mut self) -> &mut Option<Cha>;

	fn follow(&self) -> bool;

	fn from(&self) -> Url<'_>;

	async fn init(&mut self) -> anyhow::Result<Cha> {
		if self.cha().is_none() {
			*self.cha() = Some(super::File::cha(self.from(), self.follow(), None).await?)
		}
		Ok(self.cha().unwrap())
	}

	fn spawn(&self, from: UrlBuf, to: Option<UrlBuf>, cha: Cha) -> Self;

	fn to(&self) -> Option<Url<'_>>;
}

impl Traverse for FileInCopy {
	fn cha(&mut self) -> &mut Option<Cha> { &mut self.cha }

	fn follow(&self) -> bool { self.follow }

	fn from(&self) -> Url<'_> { self.from.as_url() }

	fn spawn(&self, from: UrlBuf, to: Option<UrlBuf>, cha: Cha) -> Self {
		Self {
			id: self.id,
			from,
			to: to.unwrap(),
			force: self.force,
			cha: Some(cha),
			follow: self.follow,
			retry: self.retry,
			done: self.done.clone(),
		}
	}

	fn to(&self) -> Option<Url<'_>> { Some(self.to.as_url()) }
}

impl Traverse for FileInCut {
	fn cha(&mut self) -> &mut Option<Cha> { &mut self.cha }

	fn follow(&self) -> bool { self.follow }

	fn from(&self) -> Url<'_> { self.from.as_url() }

	fn spawn(&self, from: UrlBuf, to: Option<UrlBuf>, cha: Cha) -> Self {
		Self {
			id: self.id,
			from,
			to: to.unwrap(),
			force: self.force,
			cha: Some(cha),
			follow: self.follow,
			retry: self.retry,
			done: self.done.clone(),
			drop: self.drop.clone(),
		}
	}

	fn to(&self) -> Option<Url<'_>> { Some(self.to.as_url()) }
}

impl Traverse for FileInHardlink {
	fn cha(&mut self) -> &mut Option<Cha> { &mut self.cha }

	fn follow(&self) -> bool { self.follow }

	fn from(&self) -> Url<'_> { self.from.as_url() }

	fn spawn(&self, from: UrlBuf, to: Option<UrlBuf>, cha: Cha) -> Self {
		Self {
			id: self.id,
			from,
			to: to.unwrap(),
			force: self.force,
			cha: Some(cha),
			follow: self.follow,
		}
	}

	fn to(&self) -> Option<Url<'_>> { Some(self.to.as_url()) }
}

impl Traverse for FileInDelete {
	fn cha(&mut self) -> &mut Option<Cha> { &mut self.cha }

	fn follow(&self) -> bool { false }

	fn from(&self) -> Url<'_> { self.target.as_url() }

	fn spawn(&self, from: UrlBuf, _to: Option<UrlBuf>, cha: Cha) -> Self {
		Self { id: self.id, target: from, cha: Some(cha) }
	}

	fn to(&self) -> Option<Url<'_>> { None }
}

impl Traverse for FileInDownload {
	fn cha(&mut self) -> &mut Option<Cha> { &mut self.cha }

	fn follow(&self) -> bool { true }

	fn from(&self) -> Url<'_> { self.url.as_url() }

	fn spawn(&self, from: UrlBuf, _to: Option<UrlBuf>, cha: Cha) -> Self {
		Self {
			id:    self.id,
			url:   from,
			cha:   Some(cha),
			retry: self.retry,
			done:  self.done.clone(),
		}
	}

	fn to(&self) -> Option<Url<'_>> { None }
}

impl Traverse for FileInUpload {
	fn cha(&mut self) -> &mut Option<Cha> { &mut self.cha }

	fn follow(&self) -> bool { true }

	fn from(&self) -> Url<'_> { self.url.as_url() }

	async fn init(&mut self) -> anyhow::Result<Cha> {
		if self.cha.is_none() {
			self.cha = Some(super::File::cha(self.from(), self.follow(), None).await?)
		}
		if self.cache.is_none() {
			self.cache = self.url.cache();
		}
		Ok(self.cha.unwrap())
	}

	fn spawn(&self, from: UrlBuf, _to: Option<UrlBuf>, cha: Cha) -> Self {
		Self {
			id:    self.id,
			cha:   Some(cha),
			cache: from.cache(),
			url:   from,
			done:  self.done.clone(),
		}
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
	FC: FnMut(I, Cha) -> FR,
	FR: Future<Output = Result<(), O>>,
	E: Fn(String),
{
	let cha = ctx!(task, task.init().await)?;
	if !cha.is_dir() {
		return on_file(task, cha).await;
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
		let mut it = err!(provider::read_dir(&src).await, "Cannot read directory {src:?}");

		let dest = if let Some(root) = root {
			let s = skip_url(&src, skip);
			err!(root.try_join(&s), "Cannot join {root:?} with {}", s.display())
		} else {
			src
		};

		() = err!(on_dir(dest.as_url()).await, "Cannot process directory {dest:?}");

		while let Ok(Some(entry)) = it.next().await {
			let from = entry.url();
			let cha = err!(
				super::File::cha(&from, task.follow(), Some(entry)).await,
				"Cannot get metadata for {from:?}"
			);

			if cha.is_dir() {
				dirs.push_back(from);
				continue;
			}

			let to = if root.is_some() {
				let name = from.name().unwrap();
				Some(err!(dest.try_join(name), "Cannot join {dest:?} with {}", name.display()))
			} else {
				None
			};

			err!(on_file(task.spawn(from, to, cha), cha).await, "Cannot process file");
		}
	}

	Ok(())
}
