use std::{borrow::Cow, collections::VecDeque, hash::{BuildHasher, Hash, Hasher}, path::{Path, PathBuf}};

use anyhow::{Context, Result, anyhow};
use tokio::{io::{self, ErrorKind::{AlreadyExists, NotFound}}, sync::mpsc};
use tracing::warn;
use yazi_config::YAZI;
use yazi_fs::{FsHash128, FsUrl, cha::Cha, ok_or_not_found, path::{path_relative_to, skip_url}, provider::{DirReader, FileHolder, Provider, local::Local}};
use yazi_macro::ok_or_not_found;
use yazi_shared::{timestamp_us, url::{AsUrl, Url, UrlCow, UrlLike}};
use yazi_vfs::{VfsCha, copy_with_progress, maybe_exists, provider::{self, DirEntry}, unique_name};

use super::{FileInDelete, FileInHardlink, FileInLink, FileInPaste, FileInTrash};
use crate::{LOW, NORMAL, TaskIn, TaskOp, TaskOps, file::{FileInDownload, FileInUpload, FileOutDelete, FileOutDeleteDo, FileOutDownload, FileOutDownloadDo, FileOutHardlink, FileOutHardlinkDo, FileOutLink, FileOutPaste, FileOutPasteDo, FileOutTrash, FileOutUpload, FileOutUploadDo}};

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
			return Ok(self.ops.out(task.id, FileOutPaste::Succ));
		}

		if task.cha.is_none() {
			task.cha = Some(Self::cha(&task.from, task.follow, None).await?);
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
			return Ok(self.ops.out(id, FileOutPaste::Succ));
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
				let cha = continue_unless_ok!(Self::cha(&from, task.follow, Some(entry)).await);

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

		Ok(self.ops.out(task.id, FileOutPaste::Succ))
	}

	pub(crate) async fn paste_do(&self, mut task: FileInPaste) -> Result<(), FileOutPasteDo> {
		ok_or_not_found!(provider::remove_file(&task.to).await);
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
			ok_or_not_found!(
				provider::read_link(&task.from).await,
				return Ok(self.ops.out(task.id, FileOutLink::Succ))
			)
			.into()
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

		ok_or_not_found!(provider::remove_file(&task.to).await);
		provider::symlink(&src, &task.to, async || {
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

	pub(crate) async fn hardlink(&self, mut task: FileInHardlink) -> Result<(), FileOutHardlink> {
		if task.cha.is_none() {
			task.cha = Some(Self::cha(&task.from, task.follow, None).await?);
		}

		let cha = task.cha.unwrap();
		if !cha.is_dir() {
			let id = task.id;
			self.ops.out(id, FileOutHardlink::New);
			self.queue(task, NORMAL);
			self.ops.out(id, FileOutHardlink::Succ);
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
				let cha = continue_unless_ok!(Self::cha(&from, task.follow, Some(entry)).await);

				if cha.is_dir() {
					dirs.push_back(from);
					continue;
				}

				let to = dest.join(from.name().unwrap());
				self.ops.out(task.id, FileOutHardlink::New);
				self.queue(task.spawn(from, to, cha), NORMAL);
			}
		}

		Ok(self.ops.out(task.id, FileOutHardlink::Succ))
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

	pub(crate) async fn delete(&self, mut task: FileInDelete) -> Result<(), FileOutDelete> {
		let cha = provider::symlink_metadata(&task.target).await?;
		if !cha.is_dir() {
			let id = task.id;
			task.length = cha.len;
			self.ops.out(id, FileOutDelete::New(cha.len));
			self.queue(task, NORMAL);
			self.ops.out(id, FileOutDelete::Succ);
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

		Ok(self.ops.out(task.id, FileOutDelete::Succ))
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

	pub(crate) async fn download(&self, mut task: FileInDownload) -> Result<(), FileOutDownload> {
		if task.cha.is_none() {
			task.cha = Some(Self::cha(&task.url, true, None).await?);
		}

		let cha = task.cha.unwrap();
		if cha.is_orphan() {
			Err(io::Error::new(NotFound, "Source of symlink doesn't exist"))?;
		}

		if !cha.is_dir() {
			let id = task.id;
			self.ops.out(id, FileOutDownload::New(cha.len));
			self.queue(task, LOW);
			return Ok(self.ops.out(id, FileOutDownload::Succ));
		}

		macro_rules! continue_unless_ok {
			($result:expr) => {
				match $result {
					Ok(v) => v,
					Err(e) => {
						self.ops.out(task.id, FileOutDownload::Deform(e.to_string()));
						continue;
					}
				}
			};
		}

		let mut dirs = VecDeque::from([task.url.clone()]);
		while let Some(src) = dirs.pop_front() {
			let cache = continue_unless_ok!(src.cache().ok_or("Cannot determine cache path"));
			continue_unless_ok!(match Local.create_dir(&cache).await {
				Err(e) if e.kind() != AlreadyExists => Err(e),
				_ => Ok(()),
			});

			let mut it = continue_unless_ok!(provider::read_dir(&src).await);
			while let Ok(Some(entry)) = it.next().await {
				let from = entry.url();
				let cha = continue_unless_ok!(Self::cha(&from, true, Some(entry)).await);

				if cha.is_orphan() {
					continue_unless_ok!(Err("Source of symlink doesn't exist"));
				} else if cha.is_dir() {
					dirs.push_back(from);
				} else {
					self.ops.out(task.id, FileOutDownload::New(cha.len));
					self.queue(task.spawn(from, cha), LOW);
				}
			}
		}

		Ok(self.ops.out(task.id, FileOutDownload::Succ))
	}

	pub(crate) async fn download_do(
		&self,
		mut task: FileInDownload,
	) -> Result<(), FileOutDownloadDo> {
		let cha = task.cha.unwrap();

		let cache = task.url.cache().context("Cannot determine cache path")?;
		let cache_tmp = Self::tmp(&cache).await?;

		let mut it = copy_with_progress(&task.url, Url::regular(&cache_tmp), cha);
		while let Some(res) = it.recv().await {
			match res {
				Ok(0) => {
					Local.rename(&cache_tmp, &cache).await?;

					let lock = task.url.cache_lock().context("Cannot determine cache lock")?;
					Local.write(lock, format!("{:x}", cha.hash_u128())).await?;

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

	pub(crate) async fn upload(&self, mut task: FileInUpload) -> Result<(), FileOutUpload> {
		todo!();
		// if task.cha.is_none() {
		// 	task.cha = Some(Self::cha(Url::regular(&task.path), true, None).await?);
		// }

		// let cha = task.cha.unwrap();
		// if cha.is_orphan() {
		// 	Err(io::Error::new(NotFound, "Source of symlink doesn't exist"))?;
		// }

		// if !cha.is_dir() {
		// 	let id = task.id;
		// 	self.ops.out(id, FileOutUpload::New(cha.len));
		// 	self.queue(task, LOW);
		// 	return Ok(self.ops.out(id, FileOutUpload::Succ));
		// }

		// macro_rules! continue_unless_ok {
		// 	($result:expr) => {
		// 		match $result {
		// 			Ok(v) => v,
		// 			Err(e) => {
		// 				self.ops.out(task.id, FileOutUpload::Deform(e.to_string()));
		// 				continue;
		// 			}
		// 		}
		// 	};
		// }

		// let mut dirs = VecDeque::from([task.path.clone()]);
		// while let Some(src) = dirs.pop_front() {
		// 	let cache = continue_unless_ok!(src.cache().ok_or("Cannot determine
		// cache path")); 	continue_unless_ok!(match
		// Local.create_dir(&cache).await { 		Err(e) if e.kind() != AlreadyExists
		// => Err(e), 		_ => Ok(()),
		// 	});

		// 	let mut it = continue_unless_ok!(provider::read_dir(&src).await);
		// 	while let Ok(Some(entry)) = it.next().await {
		// 		let from = entry.url();
		// 		let cha = continue_unless_ok!(Self::cha(&from, true,
		// Some(entry)).await);

		// 		if cha.is_orphan() {
		// 			continue_unless_ok!(Err("Source of symlink doesn't exist"));
		// 		} else if cha.is_dir() {
		// 			dirs.push_back(from);
		// 		} else {
		// 			self.ops.out(task.id, FileOutUpload::New(cha.len));
		// 			self.queue(task.spawn(from, cha), LOW);
		// 		}
		// 	}
		// }

		// Ok(self.ops.out(task.id, FileOutUpload::Succ))
	}

	pub(crate) async fn upload_do(&self, mut task: FileInUpload) -> Result<(), FileOutUploadDo> {
		todo!()
	}

	async fn cha<U>(url: U, follow: bool, entry: Option<DirEntry>) -> io::Result<Cha>
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

	async fn tmp(path: &Path) -> io::Result<PathBuf> {
		let Some(parent) = path.parent() else {
			Err(io::Error::new(io::ErrorKind::InvalidInput, "Path has no parent"))?
		};

		let mut h = foldhash::fast::FixedState::default().build_hasher();
		path.hash(&mut h);
		timestamp_us().hash(&mut h);

		let u = parent.join(format!(".{:x}.%tmp", h.finish())).into();
		Ok(unique_name(u, async { false }).await?.into_path().expect("a path"))
	}
}

impl File {
	#[inline]
	fn queue(&self, r#in: impl Into<TaskIn>, priority: u8) {
		_ = self.r#macro.try_send(r#in.into(), priority);
	}
}
