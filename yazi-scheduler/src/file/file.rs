use std::mem;

use anyhow::{Context, Result, anyhow};
use tokio::{io::{self, ErrorKind::NotFound}, sync::mpsc};
use yazi_fs::{Cwd, FsHash128, FsUrl, engine::{Attrs, Engine, FileHolder, local::Local}, ok_or_not_found, path::path_relative_to, stat::Stat};
use yazi_macro::{ok_or_not_found, warn};
use yazi_shared::{path::{PathCow, PathLike}, url::{AsUrl, UrlCow, UrlLike}};
use yazi_vfs::{Stamp, VfsStat, engine::{self, DirEntry}, unique_file};

use super::{FileInCopy, FileInDelete, FileInHardlink, FileInLink, FileInTrash};
use crate::{LOW, NORMAL, TaskOp, TaskOps, TasksProxy, ctx, ctx_or_not_found, file::{FileIn, FileInDownload, FileInMove, FileInUpload, FileOutCopy, FileOutCopyDo, FileOutDelete, FileOutDownload, FileOutDownloadDo, FileOutHardlink, FileOutHardlinkDo, FileOutLink, FileOutMove, FileOutMoveDo, FileOutTrash, FileOutUpload, FileOutUploadDo, Transaction, Traverse}, hook::{HookInOutCopy, HookInOutHardlink, HookInOutLink, HookInOutMove}};

pub(crate) struct File {
	ops: TaskOps,
	tx:  async_priority_channel::Sender<FileIn, u8>,
}

impl File {
	pub(crate) fn new(
		ops: &mpsc::UnboundedSender<TaskOp>,
		tx: async_priority_channel::Sender<FileIn, u8>,
	) -> Self {
		Self { ops: ops.into(), tx }
	}

	pub(crate) async fn copy(&self, mut task: FileInCopy) -> Result<(), FileOutCopy> {
		let id = task.id;

		if !task.force {
			task.to = unique_file(mem::take(&mut task.to), task.init().await?.is_dir())
				.await
				.context("Cannot determine unique destination name")?;
		}

		self.ops.out(id, HookInOutCopy::new(&task.from, &task.to));
		TasksProxy::update_succeed(id, [&task.to], true);

		super::traverse::<FileOutCopy, _, _, _, _, _>(
			task,
			async |dir| match engine::create_dir(dir).await {
				Err(e) if e.kind() != io::ErrorKind::AlreadyExists => Err(e)?,
				_ => Ok(()),
			},
			async |task, stat| {
				Ok(if stat.is_orphan() || (stat.is_indirect() && !task.follow) {
					self.ops.out(id, FileOutCopy::New(0));
					self.requeue(task.into_link(), NORMAL);
				} else {
					self.ops.out(id, FileOutCopy::New(stat.len));
					self.requeue(task, LOW);
				})
			},
			|err| {
				self.ops.out(id, FileOutCopy::Deform(err));
			},
		)
		.await?;

		Ok(self.ops.out(id, FileOutCopy::Succ))
	}

	pub(crate) async fn copy_do(&self, task: FileInCopy) -> Result<(), FileOutCopyDo> {
		ctx_or_not_found!(task, Transaction::unlink(&task.to).await);
		let mut rx = ctx!(task, engine::copy(&task.from, &task.to, task.stat.unwrap()).await)?;

		while let Some(result) = rx.recv().await {
			match result {
				Ok(n) => self.ops.out(task.id, FileOutCopyDo::Adv(n)),
				Err(e) if e.kind() == NotFound => {
					warn!("Copy task partially done: {task:?}");
					break;
				}
				Err(e) => ctx!(task, Err(e))?,
			}
		}
		Ok(self.ops.out(task.id, FileOutCopyDo::Succ))
	}

	pub(crate) async fn r#move(&self, mut task: FileInMove) -> Result<(), FileOutMove> {
		let id = task.id;

		if !task.force {
			task.to = unique_file(mem::take(&mut task.to), task.init().await?.is_dir())
				.await
				.context("Cannot determine unique destination name")?;
		}

		self.ops.out(id, HookInOutMove::new(&task.from, &task.to));
		TasksProxy::update_succeed(id, [&task.to], true);

		if !task.follow && ok_or_not_found(engine::rename(&task.from, &task.to).await).is_ok() {
			return Ok(self.ops.out(id, FileOutMove::Succ));
		}

		let (mut links, mut files) = (vec![], vec![]);
		let reorder = task.follow && ctx!(task, engine::capabilities(&task.from).await)?.symlink;

		super::traverse::<FileOutMove, _, _, _, _, _>(
			task,
			async |dir| match engine::create_dir(dir).await {
				Err(e) if e.kind() != io::ErrorKind::AlreadyExists => Err(e)?,
				_ => Ok(()),
			},
			|task, stat| {
				let nofollow = stat.is_orphan() || (stat.is_indirect() && !task.follow);
				self.ops.out(id, FileOutMove::New(if nofollow { 0 } else { stat.len }));

				if nofollow {
					self.requeue(task.into_link(), NORMAL);
				} else {
					match (stat.is_link(), reorder) {
						(_, false) => self.requeue(task, LOW),
						(true, true) => links.push(task),
						(false, true) => files.push(task),
					}
				};

				async { Ok(()) }
			},
			|err| {
				self.ops.out(id, FileOutMove::Deform(err));
			},
		)
		.await?;

		if !links.is_empty() {
			let (tx, mut rx) = mpsc::channel(1);
			for task in links {
				self.requeue(task.with_drop(&tx), LOW);
			}
			drop(tx);
			_ = rx.recv().await;
		}

		for task in files {
			self.requeue(task, LOW);
		}

		Ok(self.ops.out(id, FileOutMove::Succ))
	}

	pub(crate) async fn move_do(&self, task: FileInMove) -> Result<(), FileOutMoveDo> {
		ctx_or_not_found!(task, Transaction::unlink(&task.to).await);
		let mut rx = ctx!(task, engine::copy(&task.from, &task.to, task.stat.unwrap()).await)?;

		while let Some(result) = rx.recv().await {
			match result {
				Ok(n) => self.ops.out(task.id, FileOutMoveDo::Adv(n)),
				Err(e) if e.kind() == NotFound => {
					warn!("Move task partially done: {task:?}");
					return Ok(self.ops.out(task.id, FileOutMoveDo::Succ));
				}
				Err(e) => ctx!(task, Err(e))?,
			}
		}

		engine::remove_file(&task.from).await.ok();
		Ok(self.ops.out(task.id, FileOutMoveDo::Succ))
	}

	pub(crate) async fn link(&self, mut task: FileInLink) -> Result<(), FileOutLink> {
		if !task.force {
			task.to =
				unique_file(task.to, false).await.context("Cannot determine unique destination name")?;
		}

		self.ops.out(task.id, HookInOutLink::new(&task.from, &task.to));
		self.requeue(task, NORMAL);
		Ok(())
	}

	pub(crate) async fn link_do(&self, task: FileInLink) -> Result<(), FileOutLink> {
		let mut stat = task.stat;
		if stat.is_none() && (task.follow || task.delete) {
			stat = Some(ctx!(task, Self::stat(&task.from, task.follow, None).await)?);
		}

		let mut src: PathCow = task.from.loc().into();
		if task.follow && stat.unwrap().is_link() {
			match engine::read_link(&task.from).await {
				Ok(p) if p.is_absolute() => src = p.into(),
				Ok(p) => src = ctx!(task, task.from.loc().parent().unwrap().try_join(p))?.into(),
				Err(e) if e.kind() == io::ErrorKind::NotFound => {}
				Err(e) => ctx!(task, Err(e))?,
			}
		}

		if task.relative {
			let canon = ctx!(task, engine::canonicalize(task.to.parent().unwrap()).await)?;
			src = ctx!(task, path_relative_to(canon.loc(), src))?;
		}

		ctx_or_not_found!(task, engine::remove_file(&task.to).await);
		ctx!(
			task,
			engine::symlink(&task.to, src, async || {
				Ok(match stat {
					Some(stat) => stat.is_dir(),
					None => {
						stat = Some(Self::stat(&task.from, task.follow, None).await?);
						stat.unwrap().is_dir()
					}
				})
			})
			.await
		)?;

		if task.delete {
			let stat = stat.unwrap();
			if stat.is_dir() && stat.is_indirect() {
				engine::remove_dir(&task.from).await.ok();
			} else {
				engine::remove_file(&task.from).await.ok();
			}
		}

		Ok(self.ops.out(task.id, FileOutLink::Succ))
	}

	pub(crate) async fn hardlink(&self, mut task: FileInHardlink) -> Result<(), FileOutHardlink> {
		let id = task.id;

		if !task.force {
			task.to =
				unique_file(task.to, false).await.context("Cannot determine unique destination name")?;
		}

		self.ops.out(task.id, HookInOutHardlink::new(&task.from, &task.to));
		super::traverse::<FileOutHardlink, _, _, _, _, _>(
			task,
			async |dir| match engine::create_dir(dir).await {
				Err(e) if e.kind() != io::ErrorKind::AlreadyExists => Err(e)?,
				_ => Ok(()),
			},
			async |task, _stat| {
				self.ops.out(id, FileOutHardlink::New);
				Ok(self.requeue(task, NORMAL))
			},
			|err| {
				self.ops.out(id, FileOutHardlink::Deform(err));
			},
		)
		.await?;

		Ok(self.ops.out(id, FileOutHardlink::Succ))
	}

	pub(crate) async fn hardlink_do(&self, task: FileInHardlink) -> Result<(), FileOutHardlinkDo> {
		let src = if !task.follow {
			UrlCow::from(&task.from)
		} else if let Ok(p) = engine::canonicalize(&task.from).await {
			UrlCow::from(p)
		} else {
			UrlCow::from(&task.from)
		};

		ctx_or_not_found!(task, engine::remove_file(&task.to).await);
		ctx_or_not_found!(task, engine::hard_link(&src, &task.to).await);

		Ok(self.ops.out(task.id, FileOutHardlinkDo::Succ))
	}

	pub(crate) async fn delete(&self, task: FileInDelete) -> Result<(), FileOutDelete> {
		let result: io::Result<()> = async {
			let stat = ok_or_not_found!(engine::symlink_metadata(&task.target).await, return Ok(()));
			if stat.is_dir() && !stat.is_indirect() {
				let mut rx = engine::remove_dir_all(&task.target).await?;
				while let Some(delta) = rx.recv().await {
					self.ops.out(task.id, FileOutDelete::Adv(delta?));
				}
			} else if stat.is_dir() {
				ok_or_not_found!(engine::remove_dir(&task.target).await, return Ok(()));
				self.ops.out(task.id, FileOutDelete::Adv(1.into()));
			} else {
				ok_or_not_found!(engine::remove_file(&task.target).await, return Ok(()));
				self.ops.out(task.id, FileOutDelete::Adv(1.into()));
			}
			Ok(())
		}
		.await;

		ctx!(task, result)?;
		Ok(self.ops.out(task.id, FileOutDelete::Succ))
	}

	pub(crate) async fn trash(&self, task: FileInTrash) -> Result<(), FileOutTrash> {
		Ok(self.requeue(task, LOW))
	}

	pub(crate) async fn trash_do(&self, task: FileInTrash) -> Result<(), FileOutTrash> {
		ctx!(task, engine::trash(&task.target).await)?;
		Ok(self.ops.out(task.id, FileOutTrash::Succ))
	}

	pub(crate) async fn download(&self, task: FileInDownload) -> Result<(), FileOutDownload> {
		let id = task.id;

		super::traverse::<FileOutDownload, _, _, _, _, _>(
			task,
			async |dir| {
				let dir = dir.to_owned();
				tokio::task::spawn_blocking(move || _ = Cwd::ensure(dir.as_url())).await.ok();
				Ok(())
			},
			async |task, stat| {
				Ok(if stat.is_orphan() {
					Err(anyhow!("Failed to work on {task:?}: source of symlink doesn't exist"))?
				} else {
					self.ops.out(id, FileOutDownload::New(stat.len));
					self.requeue(task, LOW);
				})
			},
			|err| {
				self.ops.out(id, FileOutDownload::Deform(err));
			},
		)
		.await?;

		Ok(self.ops.out(id, FileOutDownload::Succ))
	}

	pub(crate) async fn download_do(&self, task: FileInDownload) -> Result<(), FileOutDownloadDo> {
		let stat = task.stat.unwrap();

		let cache = ctx!(task, task.target.cache_entry(), "Cannot determine cache path")?;
		let cache_tmp = ctx!(task, Transaction::tmp(&cache).await, "Cannot determine download cache")?;

		let mut rx = ctx!(task, engine::copy(&task.target, &cache_tmp, stat).await)?;
		while let Some(result) = rx.recv().await {
			match result {
				Ok(n) => self.ops.out(task.id, FileOutDownloadDo::Adv(n)),
				Err(e) if e.kind() == NotFound => {
					warn!("Download task partially done: {task:?}");
					return Ok(self.ops.out(task.id, FileOutDownloadDo::Succ));
				}
				Err(e) => ctx!(task, Err(e))?,
			}
		}

		if let Ok(rx) = Local::regular(&cache).remove_dir_all().await {
			rx.finish().await.ok();
		}

		ctx!(task, Stamp::write(stat, task.target.as_url()).await)?;
		ctx!(task, engine::rename(cache_tmp, cache).await, "Cannot persist downloaded file")?;
		Ok(self.ops.out(task.id, FileOutDownloadDo::Succ))
	}

	pub(crate) async fn upload(&self, task: FileInUpload) -> Result<(), FileOutUpload> {
		let id = task.id;

		super::traverse::<FileOutUpload, _, _, _, _, _>(
			task,
			async |_dir| Ok(()),
			async |task, stat| {
				let cache = ctx!(task, task.cache.as_ref(), "Cannot determine cache path")?;

				Ok(match Self::stat(cache, true, None).await {
					Ok(c) if c.mtime == stat.mtime => {}
					Ok(c) => {
						self.ops.out(id, FileOutUpload::New(c.len));
						self.requeue(task, LOW);
					}
					Err(e) if e.kind() == NotFound => {}
					Err(e) => ctx!(task, Err(e))?,
				})
			},
			|err| {
				self.ops.out(id, FileOutUpload::Deform(err));
			},
		)
		.await?;

		Ok(self.ops.out(id, FileOutUpload::Succ))
	}

	pub(crate) async fn upload_do(&self, task: FileInUpload) -> Result<(), FileOutUploadDo> {
		let stat = task.stat.unwrap();
		let cache = ctx!(task, task.cache.as_ref(), "Cannot determine cache path")?;

		let stamp = ctx!(task, Stamp::read(&task.target).await)?;
		ctx!(task, stamp.validate(stat, task.target.as_url()))?;

		let tmp =
			ctx!(task, Transaction::tmp(&task.target).await, "Cannot determine temporary upload path")?;
		let mut rx = ctx!(task, engine::copy(cache, &tmp, Attrs::mode(stat.mode)).await)?;

		while let Some(result) = rx.recv().await {
			match result {
				Ok(n) => self.ops.out(task.id, FileOutUploadDo::Adv(n)),
				Err(e) => ctx!(task, Err(e))?,
			}
		}

		let stat = ctx!(task, Self::stat(&task.target, true, None).await, "Cannot stat original file")?;
		if stamp.sig() != stat.hash_u128_str(&mut [0; 26]) {
			Err(anyhow!("Failed to work on: {task:?}: remote file has changed during upload"))?;
		}

		ctx!(task, engine::rename(&tmp, &task.target).await, "Cannot persist uploaded file")?;

		let stat = ctx!(task, Self::stat(&task.target, true, None).await, "Cannot stat uploaded file")?;
		ctx!(task, Stamp::write(stat, task.target.as_url()).await)?;

		Ok(self.ops.out(task.id, FileOutUploadDo::Succ))
	}

	pub(super) async fn stat<U>(url: U, follow: bool, dent: Option<DirEntry>) -> io::Result<Stat>
	where
		U: AsUrl,
	{
		let stat = if let Some(dent) = dent {
			dent.metadata().await?
		} else {
			engine::symlink_metadata(url.as_url()).await?
		};
		Ok(if follow { Stat::from_follow(url, stat).await } else { stat })
	}
}

impl File {
	#[inline]
	pub(crate) fn submit(&self, r#in: impl Into<FileIn>, priority: u8) {
		_ = self.tx.try_send(r#in.into(), priority);
	}

	#[inline]
	fn requeue(&self, r#in: impl Into<FileIn>, priority: u8) {
		_ = self.tx.try_send(r#in.into().into_doable(), priority);
	}
}
