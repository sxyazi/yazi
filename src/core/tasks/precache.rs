use std::path::{Path, PathBuf};

use anyhow::{Error, Result};
use image::{imageops::FilterType, ImageFormat};
use tokio::{fs, sync::mpsc};

use super::TaskOp;
use crate::{config::PREVIEW, core::external, emit};

pub struct Precache {
	rx: async_channel::Receiver<PrecacheOp>,
	tx: async_channel::Sender<PrecacheOp>,

	sch: mpsc::UnboundedSender<TaskOp>,
}

#[derive(Debug)]
pub(super) enum PrecacheOp {
	Image(PrecacheOpImage),
	Video(PrecacheOpVideo),
}

#[derive(Debug)]
pub(super) struct PrecacheOpMime {
	pub id:      usize,
	pub targets: Vec<PathBuf>,
}

#[derive(Debug)]
pub(super) struct PrecacheOpImage {
	pub id:     usize,
	pub target: PathBuf,
}

#[derive(Debug)]
pub(super) struct PrecacheOpVideo {
	pub id:     usize,
	pub target: PathBuf,
}

impl Precache {
	pub(super) fn new(sch: mpsc::UnboundedSender<TaskOp>) -> Self {
		let (tx, rx) = async_channel::unbounded();
		Self { tx, rx, sch }
	}

	#[inline]
	pub(super) async fn recv(&self) -> Result<(usize, PrecacheOp)> {
		Ok(match self.rx.recv().await? {
			PrecacheOp::Image(t) => (t.id, PrecacheOp::Image(t)),
			PrecacheOp::Video(t) => (t.id, PrecacheOp::Video(t)),
		})
	}

	pub(super) async fn work(&self, task: &mut PrecacheOp) -> Result<()> {
		match task {
			PrecacheOp::Image(task) => {
				let cache = Self::cache(&task.target);
				if fs::metadata(&cache).await.is_ok() {
					return Ok(self.sch.send(TaskOp::Adv(task.id, 1, 0))?);
				}

				let img = fs::read(&task.target).await;
				tokio::task::spawn_blocking(move || {
					let img = image::load_from_memory(&img?)?;
					let (w, h) = (PREVIEW.max_width, PREVIEW.max_height);

					if img.width() > w || img.height() > h {
						img.resize(w, h, FilterType::Triangle).save_with_format(&cache, ImageFormat::Jpeg).ok();
					}
					Ok::<(), Error>(())
				})
				.await
				.ok();

				self.sch.send(TaskOp::Adv(task.id, 1, 0))?;
			}
			PrecacheOp::Video(task) => {
				let cache = Self::cache(&task.target);
				if fs::metadata(&cache).await.is_ok() {
					return Ok(self.sch.send(TaskOp::Adv(task.id, 1, 0))?);
				}

				external::ffmpegthumbnailer(&task.target, &cache).await.ok();
				self.sch.send(TaskOp::Adv(task.id, 1, 0))?;
			}
		}
		Ok(())
	}

	fn done(&self, id: usize) -> Result<()> { Ok(self.sch.send(TaskOp::Done(id))?) }

	pub(super) async fn mime(&self, task: PrecacheOpMime) -> Result<()> {
		self.sch.send(TaskOp::New(task.id, 0))?;
		if let Ok(mimes) = external::file(&task.targets).await {
			emit!(Mimetype(mimes));
		}

		self.sch.send(TaskOp::Adv(task.id, 1, 0))?;
		self.done(task.id)
	}

	pub(super) fn image(&self, id: usize, targets: Vec<PathBuf>) -> Result<()> {
		for target in targets {
			self.sch.send(TaskOp::New(id, 0))?;
			self.tx.send_blocking(PrecacheOp::Image(PrecacheOpImage { id, target }))?;
		}
		self.done(id)
	}

	pub(super) fn video(&self, id: usize, targets: Vec<PathBuf>) -> Result<()> {
		for target in targets {
			self.sch.send(TaskOp::New(id, 0))?;
			self.tx.send_blocking(PrecacheOp::Video(PrecacheOpVideo { id, target }))?;
		}
		self.done(id)
	}

	#[inline]
	pub fn cache(path: &Path) -> PathBuf {
		format!("/tmp/yazi/{:x}", md5::compute(path.to_string_lossy().as_bytes())).into()
	}
}
