use std::mem;

use anyhow::{Context, Result, anyhow};
use tokio::{io::{self, ErrorKind::NotFound}, sync::mpsc};
use tracing::warn;
use yazi_config::YAZI;
use yazi_fs::{Cwd, FsHash128, FsUrl, cha::Cha, ok_or_not_found, path::path_relative_to, provider::{Attrs, FileHolder, Provider, local::Local}};
use yazi_shared::{path::PathCow, url::{AsUrl, UrlCow, UrlLike}};
use yazi_vfs::{VfsCha, maybe_exists, provider::{self, DirEntry}, unique_file};

use super::{FileInCopy, FileInDelete, FileInHardlink, FileInLink, FileInTrash};
use crate::{LOW, NORMAL, TaskIn, TaskOp, TaskOps, ctx, file::{FileInCut, FileInDownload, FileInUpload, FileOutCopy, FileOutCopyDo, FileOutCut, FileOutCutDo, FileOutDelete, FileOutDeleteDo, FileOutDownload, FileOutDownloadDo, FileOutHardlink, FileOutHardlinkDo, FileOutLink, FileOutTrash, FileOutUpload, FileOutUploadDo, Transaction, Traverse}, hook::{HookInOutCopy, HookInOutCut}, ok_or_not_found, progress_or_break};

pub(crate) struct File {
	ops:     TaskOps,
	r#macro: async_priority_channel::Sender<TaskIn, u8>,
}

impl File {
	pub(crate) fn new(
		ops: &mpsc::UnboundedSender<TaskOp>,
		r#macro: &async_priority_channel::Sender<TaskIn, u8>,
	) -> Self {
		Self { ops: ops.into(), r#macro: r#macro.clone() }
	}

	pub(crate) async fn copy(&self, mut task: FileInCopy) -> Result<(), FileOutCopy> {
		let id = task.id;

		if !task.force {
			task.to = unique_file(mem::take(&mut task.to), task.init().await?.is_dir())
				.await
				.context("Cannot determine unique destination name")?;
		}

		self.ops.out(id, HookInOutCopy::from(&task));
		super::traverse::<FileOutCopy, _, _, _, _, _>(
			task,
			async |dir| match provider::create_dir(dir).await {
				Err(e) if e.kind() != io::ErrorKind::AlreadyExists => Err(e)?,
				_ => Ok(()),
			},
			async |task, cha| {
				Ok(if cha.is_orphan() || (cha.is_link() && !task.follow) {
					self.ops.out(id, FileOutCopy::New(0));
					self.queue(task.into_link(), NORMAL);
				} else {
					self.ops.out(id, FileOutCopy::New(cha.len));
					self.queue(task, LOW);
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
		ok_or_not_found!(task, Transaction::unlink(&task.to).await);
		let mut it =
			ctx!(task, provider::copy_with_progress(&task.from, &task.to, task.cha.unwrap()).await)?;

		loop {
			match progress_or_break!(it, task.done) {
				Ok(0) => break,
				Ok(n) => self.ops.out(task.id, FileOutCopyDo::Adv(n)),
				Err(e) if e.kind() == NotFound => {
					warn!("Copy task partially done: {task:?}");
					break;
				}
				// Operation not permitted (os error 1)
				// Attribute not found (os error 93)
				Err(e)
					if task.retry < YAZI.tasks.bizarre_retry
						&& matches!(e.raw_os_error(), Some(1) | Some(93)) =>
				{
					task.retry += 1;
					self.ops.out(task.id, FileOutCopyDo::Log(format!("Retrying due to error: {e}")));
					return Ok(self.queue(task, LOW));
				}
				Err(e) => ctx!(task, Err(e))?,
			}
		}
		Ok(self.ops.out(task.id, FileOutCopyDo::Succ))
	}

	pub(crate) async fn cut(&self, mut task: FileInCut) -> Result<(), FileOutCut> {
		let id = task.id;

		if !task.force {
			task.to = unique_file(mem::take(&mut task.to), task.init().await?.is_dir())
				.await
				.context("Cannot determine unique destination name")?;
		}

		self.ops.out(id, HookInOutCut::from(&task));
		if !task.follow && ok_or_not_found(provider::rename(&task.from, &task.to).await).is_ok() {
			return Ok(self.ops.out(id, FileOutCut::Succ));
		}

		let (mut links, mut files) = (vec![], vec![]);
		let reorder = task.follow && ctx!(task, provider::capabilities(&task.from).await)?.symlink;

		super::traverse::<FileOutCut, _, _, _, _, _>(
			task,
			async |dir| match provider::create_dir(dir).await {
				Err(e) if e.kind() != io::ErrorKind::AlreadyExists => Err(e)?,
				_ => Ok(()),
			},
			|task, cha| {
				let nofollow = cha.is_orphan() || (cha.is_link() && !task.follow);
				self.ops.out(id, FileOutCut::New(if nofollow { 0 } else { cha.len }));

				if nofollow {
					self.queue(task.into_link(), NORMAL);
				} else {
					match (cha.is_link(), reorder) {
						(_, false) => self.queue(task, LOW),
						(true, true) => links.push(task),
						(false, true) => files.push(task),
					}
				};

				async { Ok(()) }
			},
			|err| {
				self.ops.out(id, FileOutCut::Deform(err));
			},
		)
		.await?;

		if !links.is_empty() {
			let (tx, mut rx) = mpsc::channel(1);
			for task in links {
				self.queue(task.with_drop(&tx), LOW);
			}
			drop(tx);
			while rx.recv().await.is_some() {}
		}

		for task in files {
			self.queue(task, LOW);
		}

		Ok(self.ops.out(id, FileOutCut::Succ))
	}

	pub(crate) async fn cut_do(&self, mut task: FileInCut) -> Result<(), FileOutCutDo> {
		ok_or_not_found!(task, Transaction::unlink(&task.to).await);
		let mut it =
			ctx!(task, provider::copy_with_progress(&task.from, &task.to, task.cha.unwrap()).await)?;

		loop {
			match progress_or_break!(it, task.done) {
				Ok(0) => {
					provider::remove_file(&task.from).await.ok();
					break;
				}
				Ok(n) => self.ops.out(task.id, FileOutCutDo::Adv(n)),
				Err(e) if e.kind() == NotFound => {
					warn!("Cut task partially done: {task:?}");
					break;
				}
				// Operation not permitted (os error 1)
				// Attribute not found (os error 93)
				Err(e)
					if task.retry < YAZI.tasks.bizarre_retry
						&& matches!(e.raw_os_error(), Some(1) | Some(93)) =>
				{
					task.retry += 1;
					self.ops.out(task.id, FileOutCutDo::Log(format!("Retrying due to error: {e}")));
					return Ok(self.queue(task, LOW));
				}
				Err(e) => ctx!(task, Err(e))?,
			}
		}
		Ok(self.ops.out(task.id, FileOutCutDo::Succ))
	}

	pub(crate) async fn link(&self, mut task: FileInLink) -> Result<(), FileOutLink> {
		if !task.force {
			task.to =
				unique_file(task.to, false).await.context("Cannot determine unique destination name")?;
		}

		Ok(self.queue(task, NORMAL))
	}

	pub(crate) async fn link_do(&self, task: FileInLink) -> Result<(), FileOutLink> {
		let mut src: PathCow = if task.resolve {
			ok_or_not_found!(
				task,
				provider::read_link(&task.from).await,
				return Ok(self.ops.out(task.id, FileOutLink::Succ))
			)
			.into()
		} else {
			task.from.loc().into()
		};

		if task.relative {
			let canon = ctx!(task, provider::canonicalize(task.to.parent().unwrap()).await)?;
			src = ctx!(task, path_relative_to(canon.loc(), src))?;
		}

		ok_or_not_found!(task, provider::remove_file(&task.to).await);
		ctx!(
			task,
			provider::symlink(&task.to, src, async || {
				Ok(match task.cha {
					Some(cha) => cha.is_dir(),
					None => Self::cha(&task.from, task.resolve, None).await?.is_dir(),
				})
			})
			.await
		)?;

		if task.delete {
			provider::remove_file(&task.from).await.ok();
		}
		Ok(self.ops.out(task.id, FileOutLink::Succ))
	}

	pub(crate) async fn hardlink(&self, mut task: FileInHardlink) -> Result<(), FileOutHardlink> {
		let id = task.id;

		if !task.force {
			task.to =
				unique_file(task.to, false).await.context("Cannot determine unique destination name")?;
		}

		super::traverse::<FileOutHardlink, _, _, _, _, _>(
			task,
			async |dir| match provider::create_dir(dir).await {
				Err(e) if e.kind() != io::ErrorKind::AlreadyExists => Err(e)?,
				_ => Ok(()),
			},
			async |task, _cha| {
				self.ops.out(id, FileOutHardlink::New);
				Ok(self.queue(task, NORMAL))
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
		} else if let Ok(p) = provider::canonicalize(&task.from).await {
			UrlCow::from(p)
		} else {
			UrlCow::from(&task.from)
		};

		ok_or_not_found!(task, provider::remove_file(&task.to).await);
		ok_or_not_found!(task, provider::hard_link(&src, &task.to).await);

		Ok(self.ops.out(task.id, FileOutHardlinkDo::Succ))
	}

	pub(crate) async fn delete(&self, task: FileInDelete) -> Result<(), FileOutDelete> {
		let id = task.id;

		super::traverse::<FileOutDelete, _, _, _, _, _>(
			task,
			async |_dir| Ok(()),
			async |task, cha| {
				self.ops.out(id, FileOutDelete::New(cha.len));
				Ok(self.queue(task, NORMAL))
			},
			|_err| {},
		)
		.await?;

		Ok(self.ops.out(id, FileOutDelete::Succ))
	}

	pub(crate) async fn delete_do(&self, task: FileInDelete) -> Result<(), FileOutDeleteDo> {
		match provider::remove_file(&task.target).await {
			Ok(()) => {}
			Err(e) if e.kind() == NotFound => {}
			Err(_) if !maybe_exists(&task.target).await => {}
			Err(e) => ctx!(task, Err(e))?,
		}
		Ok(self.ops.out(task.id, FileOutDeleteDo::Succ(task.cha.unwrap().len)))
	}

	pub(crate) async fn trash(&self, task: FileInTrash) -> Result<(), FileOutTrash> {
		Ok(self.queue(task, LOW))
	}

	pub(crate) async fn trash_do(&self, task: FileInTrash) -> Result<(), FileOutTrash> {
		ctx!(task, provider::trash(&task.target).await)?;
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
			async |task, cha| {
				Ok(if cha.is_orphan() {
					Err(anyhow!("Failed to work on {task:?}: source of symlink doesn't exist"))?
				} else {
					self.ops.out(id, FileOutDownload::New(cha.len));
					self.queue(task, LOW);
				})
			},
			|err| {
				self.ops.out(id, FileOutDownload::Deform(err));
			},
		)
		.await?;

		Ok(self.ops.out(id, FileOutDownload::Succ))
	}

	pub(crate) async fn download_do(
		&self,
		mut task: FileInDownload,
	) -> Result<(), FileOutDownloadDo> {
		let cha = task.cha.unwrap();

		let cache = ctx!(task, task.url.cache(), "Cannot determine cache path")?;
		let cache_tmp = ctx!(task, Transaction::tmp(&cache).await, "Cannot determine download cache")?;

		let mut it = ctx!(task, provider::copy_with_progress(&task.url, &cache_tmp, cha).await)?;
		loop {
			match progress_or_break!(it, task.done) {
				Ok(0) => {
					Local::regular(&cache).remove_dir_all().await.ok();
					ctx!(task, provider::rename(cache_tmp, cache).await, "Cannot persist downloaded file")?;

					let lock = ctx!(task, task.url.cache_lock(), "Cannot determine cache lock")?;
					let hash = format!("{:x}", cha.hash_u128());
					ctx!(task, Local::regular(&lock).write(hash).await, "Cannot lock cache")?;

					break;
				}
				Ok(n) => self.ops.out(task.id, FileOutDownloadDo::Adv(n)),
				Err(e) if e.kind() == NotFound => {
					warn!("Download task partially done: {task:?}");
					break;
				}
				// Operation not permitted (os error 1)
				// Attribute not found (os error 93)
				Err(e)
					if task.retry < YAZI.tasks.bizarre_retry
						&& matches!(e.raw_os_error(), Some(1) | Some(93)) =>
				{
					task.retry += 1;
					self.ops.out(task.id, FileOutDownloadDo::Log(format!("Retrying due to error: {e}")));
					return Ok(self.queue(task, LOW));
				}
				Err(e) => ctx!(task, Err(e))?,
			}
		}
		Ok(self.ops.out(task.id, FileOutDownloadDo::Succ))
	}

	pub(crate) async fn upload(&self, task: FileInUpload) -> Result<(), FileOutUpload> {
		let id = task.id;

		super::traverse::<FileOutUpload, _, _, _, _, _>(
			task,
			async |_dir| Ok(()),
			async |task, cha| {
				let cache = ctx!(task, task.cache.as_ref(), "Cannot determine cache path")?;

				Ok(match Self::cha(cache, true, None).await {
					Ok(c) if c.mtime == cha.mtime => {}
					Ok(c) => {
						self.ops.out(id, FileOutUpload::New(c.len));
						self.queue(task, LOW);
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
		let cha = task.cha.unwrap();
		let cache = ctx!(task, task.cache.as_ref(), "Cannot determine cache path")?;
		let lock = ctx!(task, task.url.cache_lock(), "Cannot determine cache lock")?;

		let hash = ctx!(task, Local::regular(&lock).read_to_string().await, "Cannot read cache lock")?;
		let hash = ctx!(task, u128::from_str_radix(&hash, 16), "Cannot parse hash from lock")?;
		if hash != cha.hash_u128() {
			Err(anyhow!("Failed to work on: {task:?}: remote file has changed since last download"))?;
		}

		let tmp =
			ctx!(task, Transaction::tmp(&task.url).await, "Cannot determine temporary upload path")?;
		let mut it = ctx!(
			task,
			provider::copy_with_progress(cache, &tmp, Attrs {
				mode:  Some(cha.mode),
				atime: None,
				btime: None,
				mtime: None,
			})
			.await
		)?;

		loop {
			match progress_or_break!(it, task.done) {
				Ok(0) => {
					let cha =
						ctx!(task, Self::cha(&task.url, true, None).await, "Cannot stat original file")?;
					if hash != cha.hash_u128() {
						Err(anyhow!("Failed to work on: {task:?}: remote file has changed during upload"))?;
					}

					ctx!(task, provider::rename(&tmp, &task.url).await, "Cannot persist uploaded file")?;

					let cha =
						ctx!(task, Self::cha(&task.url, true, None).await, "Cannot stat uploaded file")?;
					let hash = format!("{:x}", cha.hash_u128());
					ctx!(task, Local::regular(&lock).write(hash).await, "Cannot lock cache")?;

					break;
				}
				Ok(n) => self.ops.out(task.id, FileOutUploadDo::Adv(n)),
				Err(e) => ctx!(task, Err(e))?,
			}
		}
		Ok(self.ops.out(task.id, FileOutUploadDo::Succ))
	}

	pub(super) async fn cha<U>(url: U, follow: bool, entry: Option<DirEntry>) -> io::Result<Cha>
	where
		U: AsUrl,
	{
		let cha = if let Some(entry) = entry {
			entry.metadata().await?
		} else {
			provider::symlink_metadata(url.as_url()).await?
		};
		Ok(if follow { Cha::from_follow(url, cha).await } else { cha })
	}
}

impl File {
	#[inline]
	fn queue(&self, r#in: impl Into<TaskIn>, priority: u8) {
		_ = self.r#macro.try_send(r#in.into(), priority);
	}
}
