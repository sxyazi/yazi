use std::{collections::{BTreeMap, BTreeSet}, sync::Arc};

use anyhow::Result;
use parking_lot::Mutex;
use tokio::{fs, sync::mpsc};
use yazi_adaptor::Image;
use yazi_config::PREVIEW;
use yazi_shared::{emit, fs::{calculate_size, FilesOp, Url}, Throttle};

use crate::{external, TaskOp};

pub(crate) struct Precache {
	tx: async_channel::Sender<PrecacheOp>,
	rx: async_channel::Receiver<PrecacheOp>,

	sch: mpsc::UnboundedSender<TaskOp>,

	pub(crate) size_handing: Mutex<BTreeSet<Url>>,
}

#[derive(Debug)]
pub(crate) enum PrecacheOp {
	Image(PrecacheOpImage),
	Video(PrecacheOpVideo),
	Pdf(PrecacheOpPDF),
}

#[derive(Debug)]
pub(crate) struct PrecacheOpSize {
	pub id:       usize,
	pub target:   Url,
	pub throttle: Arc<Throttle<(Url, u64)>>,
}

#[derive(Debug)]
pub(crate) struct PrecacheOpMime {
	pub id:      usize,
	pub targets: Vec<Url>,
}

#[derive(Debug)]
pub(crate) struct PrecacheOpImage {
	pub id:     usize,
	pub target: Url,
}

#[derive(Debug)]
pub(crate) struct PrecacheOpVideo {
	pub id:     usize,
	pub target: Url,
}

#[derive(Debug)]
pub(crate) struct PrecacheOpPDF {
	pub id:     usize,
	pub target: Url,
}

impl Precache {
	pub(crate) fn new(sch: mpsc::UnboundedSender<TaskOp>) -> Self {
		let (tx, rx) = async_channel::unbounded();
		Self { tx, rx, sch, size_handing: Default::default() }
	}

	#[inline]
	pub(crate) async fn recv(&self) -> Result<(usize, PrecacheOp)> {
		Ok(match self.rx.recv().await? {
			PrecacheOp::Image(t) => (t.id, PrecacheOp::Image(t)),
			PrecacheOp::Video(t) => (t.id, PrecacheOp::Video(t)),
			PrecacheOp::Pdf(t) => (t.id, PrecacheOp::Pdf(t)),
		})
	}

	pub(crate) async fn work(&self, task: &mut PrecacheOp) -> Result<()> {
		match task {
			PrecacheOp::Image(task) => {
				let cache = PREVIEW.cache(&task.target, 0);
				if fs::symlink_metadata(&cache).await.is_ok() {
					return Ok(self.sch.send(TaskOp::Adv(task.id, 1, 0))?);
				}
				Image::precache(&task.target, cache).await.ok();
				self.sch.send(TaskOp::Adv(task.id, 1, 0))?;
			}
			PrecacheOp::Video(task) => {
				let cache = PREVIEW.cache(&task.target, 0);
				if fs::symlink_metadata(&cache).await.is_ok() {
					return Ok(self.sch.send(TaskOp::Adv(task.id, 1, 0))?);
				}

				external::ffmpegthumbnailer(&task.target, &cache, 0).await.ok();
				self.sch.send(TaskOp::Adv(task.id, 1, 0))?;
			}
			PrecacheOp::Pdf(task) => {
				let cache = PREVIEW.cache(&task.target, 0);
				if fs::symlink_metadata(&cache).await.is_ok() {
					return Ok(self.sch.send(TaskOp::Adv(task.id, 1, 0))?);
				}

				external::pdftoppm(&task.target, &cache, 0).await.ok();
				self.sch.send(TaskOp::Adv(task.id, 1, 0))?;
			}
		}
		Ok(())
	}

	pub(crate) async fn mime(&self, task: PrecacheOpMime) -> Result<()> {
		self.sch.send(TaskOp::New(task.id, 0))?;
		if let Ok(mimes) = external::file(&task.targets).await {
			emit!(Mimetype(mimes));
		}

		self.sch.send(TaskOp::Adv(task.id, 1, 0))?;
		self.succ(task.id)
	}

	pub(crate) async fn size(&self, task: PrecacheOpSize) -> Result<()> {
		self.sch.send(TaskOp::New(task.id, 0))?;

		let length = calculate_size(&task.target).await;
		task.throttle.done((task.target, length), |buf| {
			let mut handing = self.size_handing.lock();
			for (path, _) in &buf {
				handing.remove(path);
			}

			let parent = buf[0].0.parent_url().unwrap();
			emit!(Files(FilesOp::Size(parent, BTreeMap::from_iter(buf))));
		});

		self.sch.send(TaskOp::Adv(task.id, 1, 0))?;
		self.succ(task.id)
	}

	pub(crate) fn image(&self, id: usize, targets: Vec<Url>) -> Result<()> {
		for target in targets {
			self.sch.send(TaskOp::New(id, 0))?;
			self.tx.send_blocking(PrecacheOp::Image(PrecacheOpImage { id, target }))?;
		}
		self.succ(id)
	}

	pub(crate) fn video(&self, id: usize, targets: Vec<Url>) -> Result<()> {
		for target in targets {
			self.sch.send(TaskOp::New(id, 0))?;
			self.tx.send_blocking(PrecacheOp::Video(PrecacheOpVideo { id, target }))?;
		}
		self.succ(id)
	}

	pub(crate) fn pdf(&self, id: usize, targets: Vec<Url>) -> Result<()> {
		for target in targets {
			self.sch.send(TaskOp::New(id, 0))?;
			self.tx.send_blocking(PrecacheOp::Pdf(PrecacheOpPDF { id, target }))?;
		}
		self.succ(id)
	}
}

impl Precache {
	#[inline]
	fn succ(&self, id: usize) -> Result<()> { Ok(self.sch.send(TaskOp::Succ(id))?) }
}
