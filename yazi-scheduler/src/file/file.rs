use anyhow::{Context, Result, anyhow};
use tokio::{io::{self, ErrorKind::{self, NotFound}}, sync::mpsc};
use yazi_fs::{Cwd, FsHash128, FsUrl, engine::{Attrs, Engine, FileHolder, local::Local}, path::path_relative_to, stat::Stat};
use yazi_macro::{ok_or_not_found, warn};
use yazi_shared::url::{AsUrl, UrlBuf, UrlLike};
use yazi_vfs::{Stamp, VfsStat, engine::{self, DirEntry, RwFile}, unique_dir, unique_file, unique_file_with};

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

		if engine::capabilities(&task.to).await?.prepare
			&& let stat = task.init().await?
			&& let Some(u) = engine::prepare(&task.from, &task.to, stat, task.force).await?
		{
			task.to = u;
		} else if !task.force {
			task.to = if task.init().await?.is_dir() {
				unique_dir(task.to).await
			} else {
				unique_file(task.to).await.map(RwFile::into_url)
			}
			.context("Cannot determine unique destination name")?;
		}

		if task.force && ctx_or_not_found!(task, engine::same_entry(&task.from, &task.to).await) {
			return Ok(self.ops.out(id, FileOutCopy::Succ));
		}

		self.ops.out(id, HookInOutCopy::new(&task.from, &task.to));
		TasksProxy::update_succeed(id, [&task.to], true);

		super::traverse::<FileOutCopy, _, _, _, _, _>(
			task,
			async |_, dir| match engine::create_dir(dir).await {
				Err(e) if e.kind() != ErrorKind::AlreadyExists => Err(e)?,
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

	pub(crate) async fn copy_do(&self, mut task: FileInCopy) -> Result<(), FileOutCopyDo> {
		ctx_or_not_found!(task, Transaction::prepare(&task.from, &mut task.to).await);
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

		if engine::capabilities(&task.to).await?.prepare
			&& let stat = task.init().await?
			&& let Some(u) = engine::prepare(&task.from, &task.to, stat, task.force).await?
		{
			task.to = u;
		} else if !task.force {
			task.to = if task.init().await?.is_dir() {
				unique_dir(task.to).await
			} else {
				unique_file(task.to).await.map(RwFile::into_url)
			}
			.context("Cannot determine unique destination name")?;
		}

		if self.move_fast(&task).await? {
			return Ok(self.ops.out(id, FileOutMove::Succ));
		}

		let (mut links, mut files) = (vec![], vec![]);
		let reorder = task.follow && ctx!(task, engine::capabilities(&task.from).await)?.has_symlink();

		super::traverse::<FileOutMove, _, _, _, _, _>(
			task,
			async |_, dir| match engine::create_dir(dir).await {
				Err(e) if e.kind() == ErrorKind::AlreadyExists => Ok(()),
				result => result.map_err(Into::into),
			},
			|task, stat| {
				let nofollow = stat.is_orphan() || (stat.is_indirect() && !task.follow);
				self.ops.out(id, FileOutMove::New(if nofollow { 0 } else { stat.len }));

				if nofollow {
					self.requeue(task.into_link(), NORMAL);
				} else if !reorder {
					self.requeue(task, LOW);
				} else if stat.is_link() {
					links.push(task);
				} else {
					files.push(task);
				}

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

	async fn move_fast(&self, task: &FileInMove) -> Result<bool> {
		let renamed = if task.follow {
			None
		} else {
			match engine::rename(&task.from, &task.to).await {
				Err(e) if e.kind() == ErrorKind::CrossesDevices => None,
				result => Some(ctx!(task, result)?),
			}
		};

		if task.force
			&& renamed.is_none()
			&& ctx_or_not_found!(task, engine::same_entry(&task.from, &task.to).await)
		{
			return Ok(true); // Mark as succeeded if the source and destination are the same entry
		}

		let (to, hook) = match &renamed {
			Some(to) => (to, HookInOutMove::new(&task.from, to)),
			None => (&task.to, HookInOutMove::with_cleanup(&task.from, &task.to)),
		};

		self.ops.out(task.id, hook);
		TasksProxy::update_succeed(task.id, [to], true);
		Ok(renamed.is_some())
	}

	pub(crate) async fn move_do(&self, mut task: FileInMove) -> Result<(), FileOutMoveDo> {
		ctx_or_not_found!(task, Transaction::prepare(&task.from, &mut task.to).await);
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

	pub(crate) async fn link(&self, task: FileInLink) -> Result<(), FileOutLink> {
		if task.force && ctx_or_not_found!(task, engine::same_entry(&task.from, &task.to).await) {
			return Ok(self.ops.out(task.id, FileOutLink::Succ));
		}

		let file = ctx!(task, engine::file(&task.from).await)?;
		let mut target = ctx!(task, task.target(&file))?;

		if task.relative {
			let canon = ctx!(task, engine::canonicalize(task.to.parent().unwrap()).await)?;
			target = ctx!(task, path_relative_to(canon.loc(), target))?;
		}

		let target = target.into_owned();
		let create = async move |u: &UrlBuf| {
			if file.is_dir() {
				engine::symlink_dir(u, &target, task.force).await
			} else {
				engine::symlink_file(u, &target, task.force).await
			}
		};

		let (to, ()) = if task.force {
			ctx!(task, create(&task.to).await)?;
			(task.to, ())
		} else {
			ctx!(
				task,
				unique_file_with(task.to.clone(), create).await,
				"Cannot determine unique destination name"
			)?
		};

		self.ops.out(task.id, HookInOutLink::new(to));
		Ok(self.ops.out(task.id, FileOutLink::Succ))
	}

	pub(crate) async fn link_do(&self, task: FileInLink) -> Result<(), FileOutLink> {
		if ctx_or_not_found!(task, engine::same_entry(&task.from, &task.to).await) {
			return Err(anyhow!("Cannot replace the source link with itself").into());
		}

		let file = ctx!(task, engine::file(&task.from).await)?;
		let target = ctx!(task, task.target(&file))?;

		ctx!(
			task,
			if file.is_dir() {
				engine::symlink_dir(&task.to, target, true).await
			} else {
				engine::symlink_file(&task.to, target, true).await
			}
		)?;

		if task.delete {
			let stat = task.stat.unwrap_or(file.lstat());
			if stat.is_dir() && stat.is_indirect() {
				ctx_or_not_found!(task, engine::remove_dir(&task.from).await);
			} else {
				ctx_or_not_found!(task, engine::remove_file(&task.from).await);
			}
		}

		Ok(self.ops.out(task.id, FileOutLink::Succ))
	}

	pub(crate) async fn hardlink(&self, mut task: FileInHardlink) -> Result<(), FileOutHardlink> {
		let id = task.id;
		let stat = ctx!(task, task.init().await)?;
		let traverse = task.traversable(stat);

		if traverse && !task.force {
			task.to = ctx!(task, unique_dir(task.to.clone()).await)?;
		} else if traverse && ctx_or_not_found!(task, engine::same_file(&task.from, &task.to).await) {
			return Err(anyhow!("Cannot hardlink a directory onto itself").into());
		}

		if traverse || task.force {
			self.ops.out(id, HookInOutHardlink::new(&task.to));
		}

		super::traverse::<FileOutHardlink, _, _, _, _, _>(
			task,
			async |task, dir| {
				if !task.force && dir == task.to {
					return Ok(()); // Non-forced directory is already created with `unique_dir()` above.
				}

				match engine::create_dir(dir).await {
					Err(e) if e.kind() == ErrorKind::AlreadyExists => Ok(()),
					result => result.map_err(Into::into),
				}
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
		let from = task.from.clone();
		let create = async move |u: &UrlBuf| engine::hardlink(&from, u, task.follow, task.force).await;

		if task.top && !task.force {
			let (to, ()) = ctx!(
				task,
				unique_file_with(task.to.clone(), create).await,
				"Cannot determine unique destination name"
			)?;
			self.ops.out(task.id, HookInOutHardlink::new(to));
		} else {
			ctx!(task, create(&task.to).await)?;
		}

		Ok(self.ops.out(task.id, FileOutHardlinkDo::Succ))
	}

	pub(crate) async fn delete(&self, task: FileInDelete) -> Result<(), FileOutDelete> {
		let result: io::Result<()> = async {
			let stat = ok_or_not_found!(engine::lstat(&task.target).await, return Ok(()));
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
			async |_, dir| {
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
			async |_, _| Ok(()),
			async |task, stat| {
				let cache = ctx!(task, task.cache.as_ref(), "Cannot determine cache path")?;

				Ok(match Self::dstat(cache, true, None).await {
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

	pub(crate) async fn upload_do(&self, mut task: FileInUpload) -> Result<(), FileOutUploadDo> {
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

		let stat =
			ctx!(task, Self::dstat(&task.target, true, None).await, "Cannot stat original file")?;
		if stamp.sig() != stat.hash_u128_str(&mut [0; 26]) {
			Err(anyhow!("Failed to work on: {task:?}: remote file has changed during upload"))?;
		}

		task.target =
			ctx!(task, engine::rename(&tmp, &task.target).await, "Cannot persist uploaded file")?;

		let stat =
			ctx!(task, Self::dstat(&task.target, true, None).await, "Cannot stat uploaded file")?;
		ctx!(task, Stamp::write(stat, task.target.as_url()).await)?;

		Ok(self.ops.out(task.id, FileOutUploadDo::Succ(task.top.then_some(task.target))))
	}

	pub(super) async fn dstat<U>(url: U, follow: bool, dent: Option<DirEntry>) -> io::Result<Stat>
	where
		U: AsUrl,
	{
		let lstat =
			if let Some(dent) = dent { dent.lstat().await? } else { engine::lstat(&url).await? };

		Ok(if follow { Stat::from_follow(url, lstat).await } else { lstat })
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
