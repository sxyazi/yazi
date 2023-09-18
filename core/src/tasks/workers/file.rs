use std::{collections::VecDeque, fs::Metadata, path::{Path, PathBuf}};

use anyhow::Result;
use config::TASKS;
use futures::{future::BoxFuture, FutureExt};
use shared::{calculate_size, copy_with_progress, Url};
use tokio::{fs, io::{self, ErrorKind::{AlreadyExists, NotFound}}, sync::mpsc};
use tracing::trace;

use crate::tasks::TaskOp;

pub(crate) struct File {
	rx: async_channel::Receiver<FileOp>,
	tx: async_channel::Sender<FileOp>,

	sch: mpsc::UnboundedSender<TaskOp>,
}

#[derive(Debug)]
pub(crate) enum FileOp {
	Paste(FileOpPaste),
	Link(FileOpLink),
	Delete(FileOpDelete),
	Trash(FileOpTrash),
}

#[derive(Clone, Debug)]
pub(crate) struct FileOpPaste {
	pub id:     usize,
	pub from:   Url,
	pub to:     Url,
	pub cut:    bool,
	pub follow: bool,
	pub retry:  u8,
}

#[derive(Clone, Debug)]
pub(crate) struct FileOpLink {
	pub id:     usize,
	pub from:   Url,
	pub to:     Url,
	pub cut:    bool,
	pub length: u64,
}

#[derive(Clone, Debug)]
pub(crate) struct FileOpDelete {
	pub id:     usize,
	pub target: Url,
	pub length: u64,
}

#[derive(Clone, Debug)]
pub(crate) struct FileOpTrash {
	pub id:     usize,
	pub target: Url,
	pub length: u64,
}

impl File {
	pub(crate) fn new(sch: mpsc::UnboundedSender<TaskOp>) -> Self {
		let (tx, rx) = async_channel::unbounded();
		Self { tx, rx, sch }
	}

	#[inline]
	pub(crate) async fn recv(&self) -> Result<(usize, FileOp)> {
		Ok(match self.rx.recv().await? {
			FileOp::Paste(t) => (t.id, FileOp::Paste(t)),
			FileOp::Link(t) => (t.id, FileOp::Link(t)),
			FileOp::Delete(t) => (t.id, FileOp::Delete(t)),
			FileOp::Trash(t) => (t.id, FileOp::Trash(t)),
		})
	}

	pub(crate) async fn work(&self, op: &mut FileOp) -> Result<()> {
		match op {
			FileOp::Paste(task) => {
				match fs::remove_file(&task.to).await {
					Err(e) if e.kind() != NotFound => Err(e)?,
					_ => {}
				}

				let mut it = copy_with_progress(&task.from, &task.to);
				while let Some(res) = it.recv().await {
					match res {
						Ok(0) => {
							if task.cut {
								fs::remove_file(&task.from).await.ok();
							}
							break;
						}
						Ok(n) => {
							self.log(task.id, format!("Paste task advanced {n}: {:?}", task))?;
							self.sch.send(TaskOp::Adv(task.id, 0, n))?
						}
						Err(e) if e.kind() == NotFound => {
							trace!("Paste task partially done: {:?}", task);
							break;
						}
						// Operation not permitted (os error 1)
						// Attribute not found (os error 93)
						Err(e)
							if task.retry < TASKS.bizarre_retry
								&& matches!(e.raw_os_error(), Some(1) | Some(93)) =>
						{
							self.log(task.id, format!("Paste task retry: {:?}", task))?;
							task.retry += 1;
							return Ok(self.tx.send(FileOp::Paste(task.clone())).await?);
						}
						Err(e) => Err(e)?,
					}
				}
				self.sch.send(TaskOp::Adv(task.id, 1, 0))?;
			}
			FileOp::Link(task) => {
				let src = match fs::read_link(&task.from).await {
					Ok(src) => src,
					Err(e) if e.kind() == NotFound => {
						self.log(task.id, format!("Link task partially done: {:?}", task))?;
						return Ok(self.sch.send(TaskOp::Adv(task.id, 1, task.length))?);
					}
					Err(e) => Err(e)?,
				};

				match fs::remove_file(&task.to).await {
					Err(e) if e.kind() != NotFound => Err(e)?,
					_ => {
						#[cfg(target_os = "windows")]
						{
							fs::symlink_file(src, &task.to).await?
						}
						#[cfg(not(target_os = "windows"))]
						{
							fs::symlink(src, &task.to).await?
						}
					}
				}

				if task.cut {
					fs::remove_file(&task.from).await.ok();
				}
				self.sch.send(TaskOp::Adv(task.id, 1, task.length))?;
			}
			FileOp::Delete(task) => {
				if let Err(e) = fs::remove_file(&task.target).await {
					if e.kind() != NotFound && fs::symlink_metadata(&task.target).await.is_ok() {
						self.log(task.id, format!("Delete task failed: {:?}, {e}", task))?;
						Err(e)?
					}
				}
				self.sch.send(TaskOp::Adv(task.id, 1, task.length))?
			}
			FileOp::Trash(task) => {
				#[cfg(target_os = "macos")]
				{
					use trash::{macos::{DeleteMethod, TrashContextExtMacos}, TrashContext};
					let mut ctx = TrashContext::default();
					ctx.set_delete_method(DeleteMethod::NsFileManager);
					ctx.delete(&task.target)?;
				}
				#[cfg(not(target_os = "netbsd"))]
				{
					trash::delete(&task.target)?;
				}
				self.sch.send(TaskOp::Adv(task.id, 1, task.length))?;
			}
		}
		Ok(())
	}

	#[inline]
	fn log(&self, id: usize, line: String) -> Result<()> { Ok(self.sch.send(TaskOp::Log(id, line))?) }

	#[inline]
	fn done(&self, id: usize) -> Result<()> { Ok(self.sch.send(TaskOp::Done(id))?) }

	pub(crate) async fn paste(&self, mut task: FileOpPaste) -> Result<()> {
		if task.cut {
			match fs::rename(&task.from, &task.to).await {
				Ok(_) => return self.done(task.id),
				Err(e) if e.kind() == NotFound => return self.done(task.id),
				_ => {}
			}
		}

		let meta = Self::metadata(&task.from, task.follow).await?;
		if !meta.is_dir() {
			let id = task.id;
			self.sch.send(TaskOp::New(id, meta.len()))?;

			if meta.is_file() {
				self.tx.send(FileOp::Paste(task)).await?;
			} else if meta.is_symlink() {
				self.tx.send(FileOp::Link(task.to_link(meta.len()))).await?;
			}
			return self.done(id);
		}

		let root = task.to.clone();
		let skip = task.from.components().count();
		let mut dirs = VecDeque::from([task.from]);

		while let Some(src) = dirs.pop_front() {
			let dest = root.join(src.components().skip(skip).collect::<PathBuf>());
			match fs::create_dir(&dest).await {
				Err(e) if e.kind() != AlreadyExists => {
					self.log(task.id, format!("Create dir failed: {:?}, {e}", dest))?;
					continue;
				}
				_ => {}
			}

			let mut it = match fs::read_dir(&src).await {
				Ok(it) => it,
				Err(e) => {
					self.log(task.id, format!("Read dir failed: {:?}, {e}", src))?;
					continue;
				}
			};

			while let Ok(Some(entry)) = it.next_entry().await {
				let src = Url::from(entry.path());
				let Ok(meta) = Self::metadata(&src, task.follow).await else {
					continue;
				};

				if meta.is_dir() {
					dirs.push_back(src);
					continue;
				}

				task.to = dest.join(src.file_name().unwrap());
				task.from = src;
				self.sch.send(TaskOp::New(task.id, meta.len()))?;

				if meta.is_file() {
					self.tx.send(FileOp::Paste(task.clone())).await?;
				} else if meta.is_symlink() {
					self.tx.send(FileOp::Link(task.to_link(meta.len()))).await?;
				}
			}
		}
		self.done(task.id)
	}

	pub(crate) async fn delete(&self, mut task: FileOpDelete) -> Result<()> {
		let meta = fs::symlink_metadata(&task.target).await?;
		if !meta.is_dir() {
			let id = task.id;
			task.length = meta.len();
			self.sch.send(TaskOp::New(id, meta.len()))?;
			self.tx.send(FileOp::Delete(task)).await?;
			return self.done(id);
		}

		let mut dirs = VecDeque::from([task.target]);
		while let Some(target) = dirs.pop_front() {
			let mut it = match fs::read_dir(target).await {
				Ok(it) => it,
				Err(_) => continue,
			};

			while let Ok(Some(entry)) = it.next_entry().await {
				let meta = match entry.metadata().await {
					Ok(m) => m,
					Err(_) => continue,
				};

				if meta.is_dir() {
					dirs.push_front(Url::from(entry.path()));
					continue;
				}

				task.target = Url::from(entry.path());
				task.length = meta.len();
				self.sch.send(TaskOp::New(task.id, meta.len()))?;
				self.tx.send(FileOp::Delete(task.clone())).await?;
			}
		}
		self.done(task.id)
	}

	pub(crate) async fn trash(&self, mut task: FileOpTrash) -> Result<()> {
		let id = task.id;
		task.length = calculate_size(&task.target).await;

		self.sch.send(TaskOp::New(id, task.length))?;
		self.tx.send(FileOp::Trash(task)).await?;
		self.done(id)
	}

	async fn metadata(path: &Path, follow: bool) -> io::Result<Metadata> {
		if !follow {
			return fs::symlink_metadata(path).await;
		}

		let meta = fs::metadata(path).await;
		if meta.is_ok() { meta } else { fs::symlink_metadata(path).await }
	}

	pub(crate) fn remove_empty_dirs(dir: &Path) -> BoxFuture<()> {
		async move {
			let mut it = match fs::read_dir(dir).await {
				Ok(it) => it,
				Err(_) => return,
			};

			while let Ok(Some(entry)) = it.next_entry().await {
				if entry.file_type().await.map(|t| t.is_dir()).unwrap_or(false) {
					let path = entry.path();
					Self::remove_empty_dirs(&path).await;
					fs::remove_dir(path).await.ok();
				}
			}

			fs::remove_dir(dir).await.ok();
		}
		.boxed()
	}
}

impl FileOpPaste {
	fn to_link(&self, length: u64) -> FileOpLink {
		FileOpLink { id: self.id, from: self.from.clone(), to: self.to.clone(), cut: self.cut, length }
	}
}
