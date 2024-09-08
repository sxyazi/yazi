use std::collections::{HashMap, HashSet};

use anyhow::{anyhow, Result};
use parking_lot::{Mutex, RwLock};
use tokio::sync::mpsc;
use tracing::error;
use yazi_config::Priority;
use yazi_plugin::isolate;
use yazi_shared::fs::{calculate_size, FilesOp, Url};

use super::{PreworkOp, PreworkOpFetch, PreworkOpLoad, PreworkOpSize};
use crate::{TaskOp, TaskProg, HIGH, NORMAL};

pub struct Prework {
	macro_: async_priority_channel::Sender<TaskOp, u8>,
	prog:   mpsc::UnboundedSender<TaskProg>,

	pub loaded:       Mutex<HashMap<Url, u32>>,
	pub size_loading: RwLock<HashSet<Url>>,
}

impl Prework {
	pub fn new(
		macro_: async_priority_channel::Sender<TaskOp, u8>,
		prog: mpsc::UnboundedSender<TaskProg>,
	) -> Self {
		Self { macro_, prog, loaded: Default::default(), size_loading: Default::default() }
	}

	pub async fn work(&self, op: PreworkOp) -> Result<()> {
		match op {
			PreworkOp::Fetch(task) => {
				let urls: Vec<_> = task.targets.iter().map(|f| f.url_owned()).collect();
				let result = isolate::fetch(&task.plugin.name, task.targets).await;
				if let Err(e) = result {
					self.fail(
						task.id,
						format!(
							"Failed to run fetcher `{}` with:\n{}\n\nError message:\n{e}",
							task.plugin.name,
							urls.iter().map(ToString::to_string).collect::<Vec<_>>().join("\n")
						),
					)?;
					return Err(e.into());
				};

				let code = result.unwrap();
				if code & 1 == 0 {
					error!(
						"Returned {code} when running fetcher `{}` with:\n{}",
						task.plugin.name,
						urls.iter().map(ToString::to_string).collect::<Vec<_>>().join("\n")
					);
				}
				if code & 2 != 0 {
					let mut loaded = self.loaded.lock();
					for url in urls {
						loaded.get_mut(&url).map(|x| *x &= !(1 << task.plugin.id));
					}
				}
				self.prog.send(TaskProg::Adv(task.id, 1, 0))?;
			}
			PreworkOp::Load(task) => {
				let url = task.target.url_owned();
				let result = isolate::preload(&task.plugin.name, task.target).await;
				if let Err(e) = result {
					self.fail(
						task.id,
						format!("Failed to run preloader `{}` with `{url}`:\n{e}", task.plugin.name),
					)?;
					return Err(e.into());
				};

				let code = result.unwrap();
				if code & 1 == 0 {
					error!("Returned {code} when running preloader `{}` with `{url}`", task.plugin.name);
				}
				if code & 2 != 0 {
					let mut loaded = self.loaded.lock();
					loaded.get_mut(&url).map(|x| *x &= !(1 << task.plugin.id));
				}
				self.prog.send(TaskProg::Adv(task.id, 1, 0))?;
			}
			PreworkOp::Size(task) => {
				let length = calculate_size(&task.target).await;
				task.throttle.done((task.target, length), |buf| {
					{
						let mut loading = self.size_loading.write();
						for (path, _) in &buf {
							loading.remove(path);
						}
					}

					let parent = buf[0].0.parent_url().unwrap();
					FilesOp::Size(parent, HashMap::from_iter(buf)).emit();
				});
				self.prog.send(TaskProg::Adv(task.id, 1, 0))?;
			}
		}
		Ok(())
	}

	pub async fn fetch(&self, task: PreworkOpFetch) -> Result<()> {
		let id = task.id;
		self.prog.send(TaskProg::New(id, 0))?;

		match task.plugin.prio {
			Priority::Low => self.queue(PreworkOp::Fetch(task), NORMAL).await?,
			Priority::Normal => self.queue(PreworkOp::Fetch(task), HIGH).await?,
			Priority::High => self.work(PreworkOp::Fetch(task)).await?,
		}
		self.succ(id)
	}

	pub async fn load(&self, task: PreworkOpLoad) -> Result<()> {
		let id = task.id;
		self.prog.send(TaskProg::New(id, 0))?;

		match task.plugin.prio {
			Priority::Low => self.queue(PreworkOp::Load(task), NORMAL).await?,
			Priority::Normal => self.queue(PreworkOp::Load(task), HIGH).await?,
			Priority::High => self.work(PreworkOp::Load(task)).await?,
		}
		self.succ(id)
	}

	pub async fn size(&self, task: PreworkOpSize) -> Result<()> {
		let id = task.id;

		self.prog.send(TaskProg::New(id, 0))?;
		self.work(PreworkOp::Size(task)).await?;
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
	async fn queue(&self, op: impl Into<TaskOp>, priority: u8) -> Result<()> {
		self.macro_.send(op.into(), priority).await.map_err(|_| anyhow!("Failed to send task"))
	}
}
