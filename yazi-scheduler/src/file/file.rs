use std::{borrow::Cow, collections::VecDeque, path::Path};

use anyhow::{Result, anyhow};
use tokio::{fs::{self, DirEntry}, io::{self, ErrorKind::{AlreadyExists, NotFound}}, sync::mpsc};
use tracing::warn;
use yazi_config::YAZI;
use yazi_fs::{calculate_size, cha::Cha, copy_with_progress, maybe_exists, ok_or_not_found, path_relative_to, skip_path};
use yazi_shared::url::Url;

use super::{FileOp, FileOpDelete, FileOpHardlink, FileOpLink, FileOpPaste, FileOpTrash};
use crate::{LOW, NORMAL, TaskOp, TaskProg};

pub struct File {
	macro_: async_priority_channel::Sender<TaskOp, u8>,
	prog:   mpsc::UnboundedSender<TaskProg>,
}

impl File {
	pub fn new(
		macro_: async_priority_channel::Sender<TaskOp, u8>,
		prog: mpsc::UnboundedSender<TaskProg>,
	) -> Self {
		Self { macro_, prog }
	}

	pub async fn work(&self, op: FileOp) -> Result<()> {
		match op {
			FileOp::Paste(mut task) => {
				ok_or_not_found(fs::remove_file(&task.to).await)?;
				let mut it = copy_with_progress(&task.from, &task.to, task.cha.unwrap());

				while let Some(res) = it.recv().await {
					match res {
						Ok(0) => {
							if task.cut {
								fs::remove_file(&task.from).await.ok();
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
							self.log(task.id, format!("Paste task retry: {:?}", task))?;
							self.queue(FileOp::Paste(task), LOW).await?;
							return Ok(());
						}
						Err(e) => Err(e)?,
					}
				}
				self.prog.send(TaskProg::Adv(task.id, 1, 0))?;
			}
			FileOp::Link(task) => {
				let cha = task.cha.unwrap();

				let src = if task.resolve {
					match fs::read_link(&task.from).await {
						Ok(p) => Cow::Owned(p),
						Err(e) if e.kind() == NotFound => {
							warn!("Link task partially done: {task:?}");
							return Ok(self.prog.send(TaskProg::Adv(task.id, 1, cha.len))?);
						}
						Err(e) => Err(e)?,
					}
				} else {
					Cow::Borrowed(task.from.as_path())
				};

				let src = if task.relative {
					path_relative_to(&src, &fs::canonicalize(task.to.parent().unwrap()).await?)
				} else {
					src
				};

				ok_or_not_found(fs::remove_file(&task.to).await)?;
				#[cfg(unix)]
				{
					fs::symlink(src, &task.to).await?;
				}
				#[cfg(windows)]
				{
					if cha.is_dir() {
						fs::symlink_dir(src, &task.to).await?;
					} else {
						fs::symlink_file(src, &task.to).await?;
					}
				}

				if task.delete {
					fs::remove_file(&task.from).await.ok();
				}
				self.prog.send(TaskProg::Adv(task.id, 1, cha.len))?;
			}
			FileOp::Hardlink(task) => {
				let cha = task.cha.unwrap();
				let src = if !task.follow {
					Cow::Borrowed(task.from.as_path())
				} else if let Ok(p) = fs::canonicalize(&task.from).await {
					Cow::Owned(p)
				} else {
					Cow::Borrowed(task.from.as_path())
				};

				ok_or_not_found(fs::remove_file(&task.to).await)?;
				match fs::hard_link(src, &task.to).await {
					Err(e) if e.kind() == NotFound => {
						warn!("Hardlink task partially done: {task:?}");
					}
					v => v?,
				}

				self.prog.send(TaskProg::Adv(task.id, 1, cha.len))?;
			}
			FileOp::Delete(task) => {
				if let Err(e) = fs::remove_file(&task.target).await {
					if e.kind() != NotFound && maybe_exists(&task.target).await {
						self.fail(task.id, format!("Delete task failed: {:?}, {e}", task))?;
						Err(e)?
					}
				}
				self.prog.send(TaskProg::Adv(task.id, 1, task.length))?
			}
			FileOp::Trash(task) => {
				tokio::task::spawn_blocking(move || {
					#[cfg(target_os = "macos")]
					{
						use trash::{TrashContext, macos::{DeleteMethod, TrashContextExtMacos}};
						let mut ctx = TrashContext::default();
						ctx.set_delete_method(DeleteMethod::NsFileManager);
						ctx.delete(&task.target)?;
					}
					#[cfg(all(not(target_os = "macos"), not(target_os = "android")))]
					{
						trash::delete(&task.target)?;
					}
					Ok::<_, anyhow::Error>(())
				})
				.await??;
				self.prog.send(TaskProg::Adv(task.id, 1, task.length))?;
			}
		}
		Ok(())
	}

	pub async fn paste(&self, mut task: FileOpPaste) -> Result<()> {
		if task.cut && ok_or_not_found(fs::rename(&task.from, &task.to).await).is_ok() {
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
				self.queue(FileOp::Link(task.into()), NORMAL).await?;
			} else {
				self.queue(FileOp::Paste(task), LOW).await?;
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
			let dest = root.join(skip_path(&src, skip));
			continue_unless_ok!(match fs::create_dir(&dest).await {
				Err(e) if e.kind() != AlreadyExists => Err(e),
				_ => Ok(()),
			});

			let mut it = continue_unless_ok!(fs::read_dir(&src).await);
			while let Ok(Some(entry)) = it.next_entry().await {
				let from = Url::from(entry.path());
				let cha = continue_unless_ok!(Self::cha_from(entry, &from, task.follow).await);

				if cha.is_dir() {
					dirs.push_back(from);
					continue;
				}

				let to = dest.join(from.file_name().unwrap());
				self.prog.send(TaskProg::New(task.id, cha.len))?;

				if cha.is_orphan() || (cha.is_link() && !task.follow) {
					self.queue(FileOp::Link(task.spawn(from, to, cha).into()), NORMAL).await?;
				} else {
					self.queue(FileOp::Paste(task.spawn(from, to, cha)), LOW).await?;
				}
			}
		}
		self.succ(task.id)
	}

	pub async fn link(&self, mut task: FileOpLink) -> Result<()> {
		let id = task.id;
		if task.cha.is_none() {
			task.cha = Some(Self::cha(&task.from, false).await?);
		}

		self.prog.send(TaskProg::New(id, task.cha.unwrap().len))?;
		self.queue(FileOp::Link(task), NORMAL).await?;
		self.succ(id)
	}

	pub async fn hardlink(&self, mut task: FileOpHardlink) -> Result<()> {
		if task.cha.is_none() {
			task.cha = Some(Self::cha(&task.from, task.follow).await?);
		}

		let cha = task.cha.unwrap();
		if !cha.is_dir() {
			let id = task.id;
			self.prog.send(TaskProg::New(id, cha.len))?;
			self.queue(FileOp::Hardlink(task), NORMAL).await?;
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
			let dest = root.join(skip_path(&src, skip));
			continue_unless_ok!(match fs::create_dir(&dest).await {
				Err(e) if e.kind() != AlreadyExists => Err(e),
				_ => Ok(()),
			});

			let mut it = continue_unless_ok!(fs::read_dir(&src).await);
			while let Ok(Some(entry)) = it.next_entry().await {
				let from = Url::from(entry.path());
				let cha = continue_unless_ok!(Self::cha_from(entry, &from, task.follow).await);

				if cha.is_dir() {
					dirs.push_back(from);
					continue;
				}

				let to = dest.join(from.file_name().unwrap());
				self.prog.send(TaskProg::New(task.id, cha.len))?;
				self.queue(FileOp::Hardlink(task.spawn(from, to, cha)), NORMAL).await?;
			}
		}
		self.succ(task.id)
	}

	pub async fn delete(&self, mut task: FileOpDelete) -> Result<()> {
		let meta = fs::symlink_metadata(&task.target).await?;
		if !meta.is_dir() {
			let id = task.id;
			task.length = meta.len();
			self.prog.send(TaskProg::New(id, meta.len()))?;
			self.queue(FileOp::Delete(task), NORMAL).await?;
			return self.succ(id);
		}

		let mut dirs = VecDeque::from([task.target]);
		while let Some(target) = dirs.pop_front() {
			let Ok(mut it) = fs::read_dir(target).await else { continue };

			while let Ok(Some(entry)) = it.next_entry().await {
				let Ok(meta) = entry.metadata().await else { continue };

				if meta.is_dir() {
					dirs.push_front(Url::from(entry.path()));
					continue;
				}

				task.target = Url::from(entry.path());
				task.length = meta.len();
				self.prog.send(TaskProg::New(task.id, meta.len()))?;
				self.queue(FileOp::Delete(task.clone()), NORMAL).await?;
			}
		}
		self.succ(task.id)
	}

	pub async fn trash(&self, mut task: FileOpTrash) -> Result<()> {
		let id = task.id;
		task.length = calculate_size(&task.target).await;

		self.prog.send(TaskProg::New(id, task.length))?;
		self.queue(FileOp::Trash(task), LOW).await?;
		self.succ(id)
	}

	#[inline]
	async fn cha(path: &Path, follow: bool) -> io::Result<Cha> {
		let meta = fs::symlink_metadata(path).await?;
		Ok(if follow { Cha::new(path, meta).await } else { Cha::new_nofollow(path, meta) })
	}

	#[inline]
	async fn cha_from(entry: DirEntry, path: &Path, follow: bool) -> io::Result<Cha> {
		Ok(if follow {
			Cha::new(path, entry.metadata().await?).await
		} else {
			Cha::new_nofollow(path, entry.metadata().await?)
		})
	}
}

impl File {
	#[inline]
	fn succ(&self, id: usize) -> Result<()> { Ok(self.prog.send(TaskProg::Succ(id))?) }

	#[inline]
	fn fail(&self, id: usize, reason: String) -> Result<()> {
		Ok(self.prog.send(TaskProg::Fail(id, reason))?)
	}

	#[inline]
	fn log(&self, id: usize, line: String) -> Result<()> {
		Ok(self.prog.send(TaskProg::Log(id, line))?)
	}

	#[inline]
	async fn queue(&self, op: impl Into<TaskOp>, priority: u8) -> Result<()> {
		self.macro_.send(op.into(), priority).await.map_err(|_| anyhow!("Failed to send task"))
	}
}
