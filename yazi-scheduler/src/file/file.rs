use std::{borrow::Cow, collections::VecDeque};

use anyhow::{Result, anyhow, bail};
use tokio::{io::{self, ErrorKind::{AlreadyExists, NotFound}}, sync::mpsc};
use tracing::warn;
use yazi_config::YAZI;
use yazi_fs::{SizeCalculator, cha::Cha, copy_with_progress, maybe_exists, ok_or_not_found, path::{path_relative_to, skip_url}, provider::{self, DirEntry}};
use yazi_shared::{Id, url::{UrlBuf, UrlCow}};

use super::{FileIn, FileInDelete, FileInHardlink, FileInLink, FileInPaste, FileInTrash};
use crate::{LOW, NORMAL, TaskOp, TaskProg};

pub struct File {
	r#macro: async_priority_channel::Sender<TaskOp, u8>,
	prog:    mpsc::UnboundedSender<TaskProg>,
}

impl File {
	pub fn new(
		r#macro: async_priority_channel::Sender<TaskOp, u8>,
		prog: mpsc::UnboundedSender<TaskProg>,
	) -> Self {
		Self { r#macro, prog }
	}

	pub async fn work(&self, r#in: FileIn) -> Result<()> {
		match r#in {
			FileIn::Paste(mut task) => {
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
						Ok(n) => self.prog.send(TaskProg::Adv(task.id, 0, n))?,
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
							self.log(task.id, format!("Paste task retry: {task:?}"))?;
							self.queue(FileIn::Paste(task), LOW).await?;
							return Ok(());
						}
						Err(e) => Err(e)?,
					}
				}
				self.prog.send(TaskProg::Adv(task.id, 1, 0))?;
			}
			FileIn::Link(task) => {
				let cha = task.cha.unwrap();

				let src: Cow<_> = if task.resolve {
					match provider::read_link(&task.from).await {
						Ok(p) => p.into(),
						Err(e) if e.kind() == NotFound => {
							warn!("Link task partially done: {task:?}");
							return Ok(self.prog.send(TaskProg::Adv(task.id, 1, cha.len))?);
						}
						Err(e) => Err(e)?,
					}
				} else if task.from.scheme.covariant(&task.to.scheme) {
					task.from.loc.as_path().into()
				} else {
					bail!("Source and target must be on the same filesystem: {task:?}")
				};

				let src = if task.relative {
					path_relative_to(provider::canonicalize(&task.to.parent().unwrap()).await?.loc, &src)?
				} else {
					src
				};

				ok_or_not_found(provider::remove_file(&task.to).await)?;
				if cha.is_dir() {
					provider::symlink_dir(&src, &task.to).await?;
				} else {
					provider::symlink_file(&src, &task.to).await?;
				}

				if task.delete {
					provider::remove_file(&task.from).await.ok();
				}
				self.prog.send(TaskProg::Adv(task.id, 1, cha.len))?;
			}
			FileIn::Hardlink(task) => {
				let cha = task.cha.unwrap();
				let src = if !task.follow {
					UrlCow::from(&task.from)
				} else if let Ok(p) = provider::canonicalize(&task.from).await {
					UrlCow::from(p)
				} else {
					UrlCow::from(&task.from)
				};

				ok_or_not_found(provider::remove_file(&task.to).await)?;
				match provider::hard_link(&src, &task.to).await {
					Err(e) if e.kind() == NotFound => {
						warn!("Hardlink task partially done: {task:?}");
					}
					v => v?,
				}

				self.prog.send(TaskProg::Adv(task.id, 1, cha.len))?;
			}
			FileIn::Delete(task) => {
				if let Err(e) = provider::remove_file(&task.target).await
					&& e.kind() != NotFound
					&& maybe_exists(&task.target).await
				{
					self.fail(task.id, format!("Delete task failed: {task:?}, {e}"))?;
					Err(e)?
				}
				self.prog.send(TaskProg::Adv(task.id, 1, task.length))?
			}
			FileIn::Trash(task) => {
				provider::trash(&task.target).await?;
				self.prog.send(TaskProg::Adv(task.id, 1, task.length))?;
			}
		}
		Ok(())
	}

	pub async fn paste(&self, mut task: FileInPaste) -> Result<()> {
		if task.cut && ok_or_not_found(provider::rename(&task.from, &task.to).await).is_ok() {
			return self.succ(task.id);
		}

		if task.cha.is_none() {
			task.cha = Some(Self::cha(&task.from, task.follow).await?);
		}

		let cha = task.cha.unwrap();
		if !cha.is_dir() {
			let id = task.id;
			self.prog.send(TaskProg::New(id, cha.len))?;

			if cha.is_orphan() || (cha.is_link() && !task.follow) {
				self.queue(FileIn::Link(task.into()), NORMAL).await?;
			} else {
				self.queue(FileIn::Paste(task), LOW).await?;
			}
			return self.succ(id);
		}

		macro_rules! continue_unless_ok {
			($result:expr) => {
				match $result {
					Ok(v) => v,
					Err(e) => {
						self.prog.send(TaskProg::New(task.id, 0))?;
						self.fail(task.id, format!("An error occurred while pasting: {e}"))?;
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
			while let Ok(Some(entry)) = it.next_entry().await {
				let from = entry.url();
				let cha = continue_unless_ok!(Self::cha_from(entry, &from, task.follow).await);

				if cha.is_dir() {
					dirs.push_back(from);
					continue;
				}

				let to = dest.join(from.name().unwrap());
				self.prog.send(TaskProg::New(task.id, cha.len))?;

				if cha.is_orphan() || (cha.is_link() && !task.follow) {
					self.queue(FileIn::Link(task.spawn(from, to, cha).into()), NORMAL).await?;
				} else {
					self.queue(FileIn::Paste(task.spawn(from, to, cha)), LOW).await?;
				}
			}
		}
		self.succ(task.id)
	}

	pub async fn link(&self, mut task: FileInLink) -> Result<()> {
		let id = task.id;
		if task.cha.is_none() {
			task.cha = Some(Self::cha(&task.from, false).await?);
		}

		self.prog.send(TaskProg::New(id, task.cha.unwrap().len))?;
		self.queue(FileIn::Link(task), NORMAL).await?;
		self.succ(id)
	}

	pub async fn hardlink(&self, mut task: FileInHardlink) -> Result<()> {
		if task.cha.is_none() {
			task.cha = Some(Self::cha(&task.from, task.follow).await?);
		}

		let cha = task.cha.unwrap();
		if !cha.is_dir() {
			let id = task.id;
			self.prog.send(TaskProg::New(id, cha.len))?;
			self.queue(FileIn::Hardlink(task), NORMAL).await?;
			return self.succ(id);
		}

		macro_rules! continue_unless_ok {
			($result:expr) => {
				match $result {
					Ok(v) => v,
					Err(e) => {
						self.prog.send(TaskProg::New(task.id, 0))?;
						self.fail(task.id, format!("An error occurred while hardlinking: {e}"))?;
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
			while let Ok(Some(entry)) = it.next_entry().await {
				let from = entry.url();
				let cha = continue_unless_ok!(Self::cha_from(entry, &from, task.follow).await);

				if cha.is_dir() {
					dirs.push_back(from);
					continue;
				}

				let to = dest.join(from.name().unwrap());
				self.prog.send(TaskProg::New(task.id, cha.len))?;
				self.queue(FileIn::Hardlink(task.spawn(from, to, cha)), NORMAL).await?;
			}
		}
		self.succ(task.id)
	}

	pub async fn delete(&self, mut task: FileInDelete) -> Result<()> {
		let meta = provider::symlink_metadata(&task.target).await?;
		if !meta.is_dir() {
			let id = task.id;
			task.length = meta.len();
			self.prog.send(TaskProg::New(id, meta.len()))?;
			self.queue(FileIn::Delete(task), NORMAL).await?;
			return self.succ(id);
		}

		let mut dirs = VecDeque::from([task.target]);
		while let Some(target) = dirs.pop_front() {
			let Ok(mut it) = provider::read_dir(&target).await else { continue };

			while let Ok(Some(entry)) = it.next_entry().await {
				let Ok(meta) = entry.metadata().await else { continue };

				if meta.is_dir() {
					dirs.push_front(entry.url());
					continue;
				}

				task.target = entry.url();
				task.length = meta.len();
				self.prog.send(TaskProg::New(task.id, meta.len()))?;
				self.queue(FileIn::Delete(task.clone()), NORMAL).await?;
			}
		}
		self.succ(task.id)
	}

	pub async fn trash(&self, mut task: FileInTrash) -> Result<()> {
		let id = task.id;
		task.length = SizeCalculator::total(&task.target).await?;

		self.prog.send(TaskProg::New(id, task.length))?;
		self.queue(FileIn::Trash(task), LOW).await?;
		self.succ(id)
	}

	#[inline]
	async fn cha(url: &UrlBuf, follow: bool) -> io::Result<Cha> {
		let meta = provider::symlink_metadata(url).await?;
		Ok(if follow { Cha::from_follow(url, meta).await } else { Cha::new(url, meta) })
	}

	#[inline]
	async fn cha_from(entry: DirEntry, url: &UrlBuf, follow: bool) -> io::Result<Cha> {
		Ok(if follow {
			Cha::from_follow(url, entry.metadata().await?).await
		} else {
			Cha::new(url, entry.metadata().await?)
		})
	}
}

impl File {
	#[inline]
	fn succ(&self, id: Id) -> Result<()> { Ok(self.prog.send(TaskProg::Succ(id))?) }

	#[inline]
	fn fail(&self, id: Id, reason: String) -> Result<()> {
		Ok(self.prog.send(TaskProg::Fail(id, reason))?)
	}

	#[inline]
	fn log(&self, id: Id, line: String) -> Result<()> { Ok(self.prog.send(TaskProg::Log(id, line))?) }

	#[inline]
	async fn queue(&self, r#in: impl Into<TaskOp>, priority: u8) -> Result<()> {
		self.r#macro.send(r#in.into(), priority).await.map_err(|_| anyhow!("Failed to send task"))
	}
}
