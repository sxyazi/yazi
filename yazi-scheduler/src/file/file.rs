use std::{borrow::Cow, collections::VecDeque};

use anyhow::{Result, anyhow};
use tokio::{io::{self, ErrorKind::{AlreadyExists, NotFound}}, sync::mpsc};
use tracing::warn;
use yazi_config::YAZI;
use yazi_fs::{cha::Cha, copy_with_progress, maybe_exists, ok_or_not_found, path::{path_relative_to, skip_url}, provider::{self, DirEntry, DirReader, FileHolder}};
use yazi_shared::url::{Url, UrlBuf, UrlCow};

use super::{FileInDelete, FileInHardlink, FileInLink, FileInPaste, FileInTrash};
use crate::{LOW, NORMAL, TaskIn, TaskOp, TaskOps, file::{FileOutDelete, FileOutDeleteDo, FileOutHardlink, FileOutHardlinkDo, FileOutLink, FileOutPaste, FileOutPasteDo, FileOutTrash}};

pub(crate) struct File {
	ops:     TaskOps,
	r#macro: async_priority_channel::Sender<TaskIn, u8>,
}

impl File {
	pub(crate) fn new(
		tx: &mpsc::UnboundedSender<TaskOp>,
		r#macro: &async_priority_channel::Sender<TaskIn, u8>,
	) -> Self {
		Self { ops: tx.into(), r#macro: r#macro.clone() }
	}

	pub(crate) async fn paste(&self, mut task: FileInPaste) -> Result<(), FileOutPaste> {
		if task.cut && ok_or_not_found(provider::rename(&task.from, &task.to).await).is_ok() {
			return Ok(self.ops.out(task.id, FileOutPaste::Init));
		}

		if task.cha.is_none() {
			task.cha = Some(Self::cha(&task.from, task.follow).await?);
		}

		let cha = task.cha.unwrap();
		if !cha.is_dir() {
			let id = task.id;
			if cha.is_orphan() || (cha.is_link() && !task.follow) {
				self.ops.out(id, FileOutPaste::New(0));
				self.queue(task.into_link(), NORMAL);
			} else {
				self.ops.out(id, FileOutPaste::New(cha.len));
				self.queue(task, LOW);
			}
			return Ok(self.ops.out(id, FileOutPaste::Init));
		}

		macro_rules! continue_unless_ok {
			($result:expr) => {
				match $result {
					Ok(v) => v,
					Err(e) => {
						self.ops.out(task.id, FileOutPaste::Deform(e.to_string()));
						continue;
					}
				}
			};
		}

		let root = &task.to;
		let skip = task.from.components().count();
		let mut dirs = VecDeque::from([task.from.clone()]);

		while let Some(src) = dirs.pop_front() {
			let dest = root.join(skip_url(&src, skip));
			continue_unless_ok!(match provider::create_dir(&dest).await {
				Err(e) if e.kind() != AlreadyExists => Err(e),
				_ => Ok(()),
			});

			let mut it = continue_unless_ok!(provider::read_dir(&src).await);
			while let Ok(Some(entry)) = it.next().await {
				let from = entry.url();
				let cha = continue_unless_ok!(Self::entry_cha(entry, &from, task.follow).await);

				if cha.is_dir() {
					dirs.push_back(from);
					continue;
				}

				let to = dest.join(from.name().unwrap());
				if cha.is_orphan() || (cha.is_link() && !task.follow) {
					self.ops.out(task.id, FileOutPaste::New(0));
					self.queue(task.spawn(from, to, cha).into_link(), NORMAL);
				} else {
					self.ops.out(task.id, FileOutPaste::New(cha.len));
					self.queue(task.spawn(from, to, cha), LOW);
				}
			}
		}

		Ok(self.ops.out(task.id, FileOutPaste::Init))
	}

	pub(crate) async fn paste_do(&self, mut task: FileInPaste) -> Result<(), FileOutPasteDo> {
		ok_or_not_found(provider::remove_file(&task.to).await)?;
		let mut it = copy_with_progress(&task.from, &task.to, task.cha.unwrap());

		while let Some(res) = it.recv().await {
			match res {
				Ok(0) => {
					if task.cut {
						provider::remove_file(&task.from).await.ok();
					}
					break;
				}
				Ok(n) => self.ops.out(task.id, FileOutPasteDo::Adv(n)),
				Err(e) if e.kind() == NotFound => {
					warn!("Paste task partially done: {task:?}");
					break;
				}
				// Operation not permitted (os error 1)
				// Attribute not found (os error 93)
				Err(e)
					if task.retry < YAZI.tasks.bizarre_retry
						&& matches!(e.raw_os_error(), Some(1) | Some(93)) =>
				{
					task.retry += 1;
					self.ops.out(task.id, FileOutPasteDo::Log(format!("Retrying due to error: {e}")));
					return Ok(self.queue(task, LOW));
				}
				Err(e) => Err(e)?,
			}
		}
		Ok(self.ops.out(task.id, FileOutPasteDo::Succ))
	}

	pub(crate) fn link(&self, task: FileInLink) -> Result<(), FileOutLink> {
		Ok(self.queue(task, NORMAL))
	}

	pub(crate) async fn link_do(&self, task: FileInLink) -> Result<(), FileOutLink> {
		let src: Cow<_> = if task.resolve {
			match provider::read_link(&task.from).await {
				Ok(p) => p.into(),
				Err(e) if e.kind() == NotFound => {
					return Ok(self.ops.out(task.id, FileOutLink::Succ));
				}
				Err(e) => Err(e)?,
			}
		} else if task.from.scheme.covariant(&task.to.scheme) {
			task.from.loc.as_path().into()
		} else {
			Err(anyhow!("Source and target must be on the same filesystem: {task:?}"))?
		};

		let src = if task.relative {
			path_relative_to(provider::canonicalize(task.to.parent().unwrap()).await?.loc, &src)?
		} else {
			src
		};

		ok_or_not_found(provider::remove_file(&task.to).await)?;
		provider::symlink(&src, &task.to, async || {
			Ok(match task.cha {
				Some(cha) => cha.is_dir(),
				None => Self::cha(&task.from, task.resolve).await?.is_dir(),
			})
		})
		.await?;

		if task.delete {
			provider::remove_file(&task.from).await.ok();
		}
		Ok(self.ops.out(task.id, FileOutLink::Succ))
	}

	pub(crate) async fn hardlink(&self, mut task: FileInHardlink) -> Result<(), FileOutHardlink> {
		if task.cha.is_none() {
			task.cha = Some(Self::cha(&task.from, task.follow).await?);
		}

		let cha = task.cha.unwrap();
		if !cha.is_dir() {
			let id = task.id;
			self.ops.out(id, FileOutHardlink::New);
			self.queue(task, NORMAL);
			self.ops.out(id, FileOutHardlink::Init);
			return Ok(());
		}

		macro_rules! continue_unless_ok {
			($result:expr) => {
				match $result {
					Ok(v) => v,
					Err(e) => {
						self.ops.out(task.id, FileOutHardlink::Deform(e.to_string()));
						continue;
					}
				}
			};
		}

		let root = &task.to;
		let skip = task.from.components().count();
		let mut dirs = VecDeque::from([task.from.clone()]);

		while let Some(src) = dirs.pop_front() {
			let dest = root.join(skip_url(&src, skip));
			continue_unless_ok!(match provider::create_dir(&dest).await {
				Err(e) if e.kind() != AlreadyExists => Err(e),
				_ => Ok(()),
			});

			let mut it = continue_unless_ok!(provider::read_dir(&src).await);
			while let Ok(Some(entry)) = it.next().await {
				let from = entry.url();
				let cha = continue_unless_ok!(Self::entry_cha(entry, &from, task.follow).await);

				if cha.is_dir() {
					dirs.push_back(from);
					continue;
				}

				let to = dest.join(from.name().unwrap());
				self.ops.out(task.id, FileOutHardlink::New);
				self.queue(task.spawn(from, to, cha), NORMAL);
			}
		}

		Ok(self.ops.out(task.id, FileOutHardlink::Init))
	}

	pub(crate) async fn hardlink_do(&self, task: FileInHardlink) -> Result<(), FileOutHardlinkDo> {
		let src = if !task.follow {
			UrlCow::from(&task.from)
		} else if let Ok(p) = provider::canonicalize(&task.from).await {
			UrlCow::from(p)
		} else {
			UrlCow::from(&task.from)
		};

		ok_or_not_found(provider::remove_file(&task.to).await)?;
		ok_or_not_found(provider::hard_link(&src, &task.to).await)?;

		Ok(self.ops.out(task.id, FileOutHardlinkDo::Succ))
	}

	pub(crate) async fn delete(&self, mut task: FileInDelete) -> Result<(), FileOutDelete> {
		let cha = provider::symlink_metadata(&task.target).await?;
		if !cha.is_dir() {
			let id = task.id;
			task.length = cha.len;
			self.ops.out(id, FileOutDelete::New(cha.len));
			self.queue(task, NORMAL);
			self.ops.out(id, FileOutDelete::Init);
			return Ok(());
		}

		let mut dirs = VecDeque::from([task.target]);
		while let Some(target) = dirs.pop_front() {
			let Ok(mut it) = provider::read_dir(&target).await else { continue };

			while let Ok(Some(entry)) = it.next().await {
				let Ok(cha) = entry.metadata().await else { continue };

				if cha.is_dir() {
					dirs.push_front(entry.url());
					continue;
				}

				task.target = entry.url();
				task.length = cha.len;
				self.ops.out(task.id, FileOutDelete::New(cha.len));
				self.queue(task.clone(), NORMAL);
			}
		}

		Ok(self.ops.out(task.id, FileOutDelete::Init))
	}

	pub(crate) async fn delete_do(&self, task: FileInDelete) -> Result<(), FileOutDeleteDo> {
		match provider::remove_file(&task.target).await {
			Ok(()) => {}
			Err(e) if e.kind() == NotFound => {}
			Err(_) if !maybe_exists(&task.target).await => {}
			Err(e) => Err(e)?,
		}
		Ok(self.ops.out(task.id, FileOutDeleteDo::Succ(task.length)))
	}

	pub(crate) fn trash(&self, task: FileInTrash) -> Result<(), FileOutTrash> {
		Ok(self.queue(task, LOW))
	}

	pub(crate) async fn trash_do(&self, task: FileInTrash) -> Result<(), FileOutTrash> {
		provider::trash(&task.target).await?;
		Ok(self.ops.out(task.id, FileOutTrash::Succ))
	}

	#[inline]
	async fn cha<'a>(url: impl Into<Url<'a>>, follow: bool) -> io::Result<Cha> {
		let url = url.into();
		let cha = provider::symlink_metadata(url).await?;
		Ok(if follow { Cha::from_follow(url, cha).await } else { cha })
	}

	#[inline]
	async fn entry_cha(entry: DirEntry, url: &UrlBuf, follow: bool) -> io::Result<Cha> {
		Ok(if follow {
			Cha::from_follow(url, entry.metadata().await?).await
		} else {
			entry.metadata().await?
		})
	}
}

impl File {
	#[inline]
	fn queue(&self, r#in: impl Into<TaskIn>, priority: u8) {
		_ = self.r#macro.try_send(r#in.into(), priority);
	}
}
