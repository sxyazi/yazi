use std::hash::{BuildHasher, Hash, Hasher};

use anyhow::{Context, Result, anyhow};
use tokio::{io::{self, ErrorKind::{AlreadyExists, NotFound}}, sync::mpsc};
use tracing::warn;
use yazi_config::YAZI;
use yazi_fs::{Cwd, FsHash128, FsUrl, cha::Cha, ok_or_not_found, path::path_relative_to, provider::{Attrs, FileHolder, Provider, local::Local}};
use yazi_macro::ok_or_not_found;
use yazi_shared::{path::PathCow, timestamp_us, url::{AsUrl, UrlBuf, UrlCow, UrlLike}};
use yazi_vfs::{VfsCha, maybe_exists, provider::{self, DirEntry}, unique_name};

use super::{FileInCopy, FileInDelete, FileInHardlink, FileInLink, FileInTrash};
use crate::{LOW, NORMAL, TaskIn, TaskOp, TaskOps, file::{FileInCut, FileInDownload, FileInUpload, FileOutCopy, FileOutCopyDo, FileOutCut, FileOutCutDo, FileOutDelete, FileOutDeleteDo, FileOutDownload, FileOutDownloadDo, FileOutHardlink, FileOutHardlinkDo, FileOutLink, FileOutTrash, FileOutUpload, FileOutUploadDo}};

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

	pub(crate) async fn copy(&self, task: FileInCopy) -> Result<(), FileOutCopy> {
		let id = task.id;

		super::traverse::<FileOutCopy, _, _, _, _, _>(
			task,
			async |dir| match provider::create_dir(dir).await {
				Err(e) if e.kind() != AlreadyExists => Err(e)?,
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
		ok_or_not_found!(provider::remove_file(&task.to).await);
		let mut it = provider::copy_with_progress(&task.from, &task.to, task.cha.unwrap()).await?;

		while let Some(res) = it.recv().await {
			match res {
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
				Err(e) => Err(e)?,
			}
		}
		Ok(self.ops.out(task.id, FileOutCopyDo::Succ))
	}

	pub(crate) async fn cut(&self, task: FileInCut) -> Result<(), FileOutCut> {
		let id = task.id;

		if !task.follow && ok_or_not_found(provider::rename(&task.from, &task.to).await).is_ok() {
			return Ok(self.ops.out(id, FileOutCut::Succ));
		}

		let (mut links, mut files) = (vec![], vec![]);
		let reorder = task.follow && provider::capabilities(&task.from).await?.symlink;

		super::traverse::<FileOutCut, _, _, _, _, _>(
			task,
			async |dir| match provider::create_dir(dir).await {
				Err(e) if e.kind() != AlreadyExists => Err(e)?,
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
			let len = links.len();
			let (tx, mut rx) = mpsc::channel(len);
			for task in links {
				self.queue(task.with_drop(&tx), LOW);
			}
			for _ in 0..len {
				rx.recv().await;
			}
		}

		for task in files {
			self.queue(task, LOW);
		}

		Ok(self.ops.out(id, FileOutCut::Succ))
	}

	pub(crate) async fn cut_do(&self, mut task: FileInCut) -> Result<(), FileOutCutDo> {
		ok_or_not_found!(provider::remove_file(&task.to).await);
		let mut it = provider::copy_with_progress(&task.from, &task.to, task.cha.unwrap()).await?;

		while let Some(res) = it.recv().await {
			match res {
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
				Err(e) => Err(e)?,
			}
		}
		Ok(self.ops.out(task.id, FileOutCutDo::Succ))
	}

	pub(crate) fn link(&self, task: FileInLink) -> Result<(), FileOutLink> {
		Ok(self.queue(task, NORMAL))
	}

	pub(crate) async fn link_do(&self, task: FileInLink) -> Result<(), FileOutLink> {
		let mut src: PathCow = if task.resolve {
			ok_or_not_found!(
				provider::read_link(&task.from).await,
				return Ok(self.ops.out(task.id, FileOutLink::Succ))
			)
			.into()
		} else {
			task.from.loc().into()
		};

		if task.relative {
			let canon = provider::canonicalize(task.to.parent().unwrap()).await?;
			src = path_relative_to(canon.loc(), src)?;
		}

		ok_or_not_found!(provider::remove_file(&task.to).await);
		provider::symlink(task.to, src, async || {
			Ok(match task.cha {
				Some(cha) => cha.is_dir(),
				None => Self::cha(&task.from, task.resolve, None).await?.is_dir(),
			})
		})
		.await?;

		if task.delete {
			provider::remove_file(&task.from).await.ok();
		}
		Ok(self.ops.out(task.id, FileOutLink::Succ))
	}

	pub(crate) async fn hardlink(&self, task: FileInHardlink) -> Result<(), FileOutHardlink> {
		let id = task.id;

		super::traverse::<FileOutHardlink, _, _, _, _, _>(
			task,
			async |dir| match provider::create_dir(dir).await {
				Err(e) if e.kind() != AlreadyExists => Err(e)?,
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

		ok_or_not_found!(provider::remove_file(&task.to).await);
		ok_or_not_found!(provider::hard_link(&src, &task.to).await);

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
			Err(e) => Err(e)?,
		}
		Ok(self.ops.out(task.id, FileOutDeleteDo::Succ(task.cha.unwrap().len)))
	}

	pub(crate) fn trash(&self, task: FileInTrash) -> Result<(), FileOutTrash> {
		Ok(self.queue(task, LOW))
	}

	pub(crate) async fn trash_do(&self, task: FileInTrash) -> Result<(), FileOutTrash> {
		provider::trash(&task.target).await?;
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
					Err(io::Error::new(NotFound, "Source of symlink doesn't exist"))?;
				} else {
					self.ops.out(id, FileOutDownload::New(cha.len));
					self.queue(task, LOW)
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

		let cache = task.url.cache().context("Cannot determine cache path")?;
		let cache_tmp = Self::tmp(&cache).await.context("Cannot determine temporary download cache")?;

		let mut it = provider::copy_with_progress(&task.url, &cache_tmp, cha).await?;
		while let Some(res) = it.recv().await {
			match res {
				Ok(0) => {
					Local::regular(&cache).remove_dir_all().await.ok();
					provider::rename(cache_tmp, cache).await.context("Cannot persist downloaded file")?;

					let lock = task.url.cache_lock().context("Cannot determine cache lock")?;
					let hash = format!("{:x}", cha.hash_u128());
					Local::regular(&lock).write(hash).await.context("Cannot lock cache")?;

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
				Err(e) => Err(e)?,
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
				let cache = task.cache.as_ref().context("Cannot determine cache path")?;

				Ok(match Self::cha(cache, true, None).await {
					Ok(c) if c.mtime == cha.mtime => {}
					Ok(c) => {
						self.ops.out(id, FileOutUpload::New(c.len));
						self.queue(task, LOW);
					}
					Err(e) if e.kind() == NotFound => {}
					Err(e) => Err(e)?,
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
		let cache = task.cache.context("Cannot determine cache path")?;
		let lock = task.url.cache_lock().context("Cannot determine cache lock")?;

		let hash = Local::regular(&lock).read_to_string().await.context("Cannot read cache lock")?;
		let hash = u128::from_str_radix(&hash, 16).context("Cannot parse hash from lock")?;
		if hash != cha.hash_u128() {
			Err(anyhow!("Remote file has changed since last download"))?;
		}

		let tmp = Self::tmp(&task.url).await.context("Cannot determine temporary upload path")?;
		let mut it = provider::copy_with_progress(&cache, &tmp, Attrs {
			mode:  Some(cha.mode),
			atime: None,
			btime: None,
			mtime: None,
		})
		.await?;

		while let Some(res) = it.recv().await {
			match res {
				Ok(0) => {
					let cha = Self::cha(&task.url, true, None).await.context("Cannot stat original file")?;
					if hash != cha.hash_u128() {
						Err(anyhow!("Remote file has changed during upload"))?;
					}

					provider::rename(&tmp, &task.url).await.context("Cannot persist uploaded file")?;

					let cha = Self::cha(&task.url, true, None).await.context("Cannot stat uploaded file")?;
					let hash = format!("{:x}", cha.hash_u128());
					Local::regular(&lock).write(hash).await.context("Cannot lock cache")?;

					break;
				}
				Ok(n) => self.ops.out(task.id, FileOutUploadDo::Adv(n)),
				Err(e) => Err(e)?,
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

	async fn tmp<U>(url: U) -> io::Result<UrlBuf>
	where
		U: AsUrl,
	{
		let url = url.as_url();
		let Some(parent) = url.parent() else {
			Err(io::Error::new(io::ErrorKind::InvalidInput, "Url has no parent"))?
		};

		let mut h = foldhash::fast::FixedState::default().build_hasher();
		url.hash(&mut h);
		timestamp_us().hash(&mut h);

		unique_name(parent.try_join(format!(".{:x}.%tmp", h.finish()))?, async { false }).await
	}
}

impl File {
	#[inline]
	fn queue(&self, r#in: impl Into<TaskIn>, priority: u8) {
		_ = self.r#macro.try_send(r#in.into(), priority);
	}
}
