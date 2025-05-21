use std::{collections::{HashMap, HashSet}, num::NonZeroUsize};

use anyhow::{Result, anyhow};
use lru::LruCache;
use parking_lot::{Mutex, RwLock};
use tokio::sync::mpsc;
use tracing::error;
use yazi_config::Priority;
use yazi_fs::{FilesOp, SizeCalculator};
use yazi_plugin::isolate;
use yazi_shared::{event::CmdCow, url::Url};

use super::{PreworkIn, PreworkInFetch, PreworkInLoad, PreworkInSize};
use crate::{HIGH, NORMAL, TaskOp, TaskProg};

pub struct Prework {
	r#macro: async_priority_channel::Sender<TaskOp, u8>,
	prog:    mpsc::UnboundedSender<TaskProg>,

	pub loaded:       Mutex<LruCache<u64, u32>>,
	pub size_loading: RwLock<HashSet<Url>>,
}

impl Prework {
	pub fn new(
		r#macro: async_priority_channel::Sender<TaskOp, u8>,
		prog: mpsc::UnboundedSender<TaskProg>,
	) -> Self {
		Self {
			r#macro,
			prog,
			loaded: Mutex::new(LruCache::new(NonZeroUsize::new(4096).unwrap())),
			size_loading: Default::default(),
		}
	}

	pub async fn work(&self, r#in: PreworkIn) -> Result<()> {
		match r#in {
			PreworkIn::Fetch(task) => {
				let hashes: Vec<_> = task.targets.iter().map(|f| f.hash()).collect();
				let result = isolate::fetch(CmdCow::from(&task.plugin.run), task.targets).await;
				if let Err(e) = result {
					self.fail(task.id, format!("Failed to run fetcher `{}`:\n{e}", task.plugin.run.name))?;
					return Err(e.into());
				};

				let (state, err) = result.unwrap();
				let mut loaded = self.loaded.lock();
				for (_, h) in hashes.into_iter().enumerate().filter(|&(i, _)| !state.get(i)) {
					loaded.get_mut(&h).map(|x| *x &= !(1 << task.plugin.idx));
				}
				if let Some(e) = err {
					error!("Error when running fetcher `{}`:\n{e}", task.plugin.run.name);
				}
				self.prog.send(TaskProg::Adv(task.id, 1, 0))?;
			}
			PreworkIn::Load(task) => {
				let hash = task.target.hash();
				let result = isolate::preload(&task.plugin.run, task.target).await;
				if let Err(e) = result {
					self
						.fail(task.id, format!("Failed to run preloader `{}`:\n{e}", task.plugin.run.name))?;
					return Err(e.into());
				};

				let (ok, err) = result.unwrap();
				if !ok {
					self.loaded.lock().get_mut(&hash).map(|x| *x &= !(1 << task.plugin.idx));
				}
				if let Some(e) = err {
					error!("Error when running preloader `{}`:\n{e}", task.plugin.run.name);
				}
				self.prog.send(TaskProg::Adv(task.id, 1, 0))?;
			}
			PreworkIn::Size(task) => {
				let length = SizeCalculator::total(&task.target).await.unwrap_or(0);
				task.throttle.done((task.target, length), |buf| {
					{
						let mut loading = self.size_loading.write();
						for (path, _) in &buf {
							loading.remove(path);
						}
					}

					let parent = buf[0].0.parent_url().unwrap();
					FilesOp::Size(
						parent,
						HashMap::from_iter(buf.into_iter().map(|(u, s)| (u.urn_owned(), s))),
					)
					.emit();
				});
				self.prog.send(TaskProg::Adv(task.id, 1, 0))?;
			}
		}
		Ok(())
	}

	pub async fn fetch(&self, task: PreworkInFetch) -> Result<()> {
		let id = task.id;
		self.prog.send(TaskProg::New(id, 0))?;

		match task.plugin.prio {
			Priority::Low => self.queue(PreworkIn::Fetch(task), NORMAL).await?,
			Priority::Normal => self.queue(PreworkIn::Fetch(task), HIGH).await?,
			Priority::High => self.work(PreworkIn::Fetch(task)).await?,
		}
		self.succ(id)
	}

	pub async fn load(&self, task: PreworkInLoad) -> Result<()> {
		let id = task.id;
		self.prog.send(TaskProg::New(id, 0))?;

		match task.plugin.prio {
			Priority::Low => self.queue(PreworkIn::Load(task), NORMAL).await?,
			Priority::Normal => self.queue(PreworkIn::Load(task), HIGH).await?,
			Priority::High => self.work(PreworkIn::Load(task)).await?,
		}
		self.succ(id)
	}

	pub async fn size(&self, task: PreworkInSize) -> Result<()> {
		let id = task.id;

		self.prog.send(TaskProg::New(id, 0))?;
		self.work(PreworkIn::Size(task)).await?;
		self.succ(id)
	}
}

impl Prework {
	#[inline]
	fn succ(&self, id: usize) -> Result<()> { Ok(self.prog.send(TaskProg::Succ(id))?) }

	#[inline]
	fn fail(&self, id: usize, reason: String) -> Result<()> {
		Ok(self.prog.send(TaskProg::Fail(id, reason))?)
	}

	#[inline]
	async fn queue(&self, r#in: impl Into<TaskOp>, priority: u8) -> Result<()> {
		self.r#macro.send(r#in.into(), priority).await.map_err(|_| anyhow!("Failed to send task"))
	}
}
