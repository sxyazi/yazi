use std::collections::{HashMap, HashSet};

use anyhow::Result;
use parking_lot::RwLock;
use tokio::sync::mpsc;
use tracing::error;
use yazi_config::Priority;
use yazi_plugin::isolate;
use yazi_shared::fs::{calculate_size, FilesOp, Url};

use super::{PreloadOp, PreloadOpRule, PreloadOpSize};
use crate::{TaskOp, TaskProg, HIGH, NORMAL};

pub struct Preload {
	macro_: async_priority_channel::Sender<TaskOp, u8>,
	prog:   mpsc::UnboundedSender<TaskProg>,

	pub rule_loaded:  RwLock<HashMap<Url, u32>>,
	pub size_loading: RwLock<HashSet<Url>>,
}

impl Preload {
	pub fn new(
		macro_: async_priority_channel::Sender<TaskOp, u8>,
		prog: mpsc::UnboundedSender<TaskProg>,
	) -> Self {
		Self { macro_, prog, rule_loaded: Default::default(), size_loading: Default::default() }
	}

	pub async fn work(&self, op: PreloadOp) -> Result<()> {
		match op {
			PreloadOp::Rule(task) => {
				let urls: Vec<_> = task.targets.iter().map(|f| f.url()).collect();
				let result = isolate::preload(&task.plugin.name, task.targets, task.plugin.multi).await;
				if let Err(e) = result {
					self.fail(task.id, format!("Preload task failed:\n{e}"))?;
					return Err(e.into());
				};

				let code = result.unwrap();
				if code & 1 == 0 {
					error!("Preload task `{}` returned {code}", task.plugin.name);
				}
				if code >> 1 & 1 != 0 {
					let mut loaded = self.rule_loaded.write();
					for url in urls {
						loaded.get_mut(&url).map(|x| *x ^= 1 << task.plugin.id);
					}
				}
				self.prog.send(TaskProg::Adv(task.id, 1, 0))?;
			}
			PreloadOp::Size(task) => {
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

	pub async fn rule(&self, task: PreloadOpRule) -> Result<()> {
		let id = task.id;
		self.prog.send(TaskProg::New(id, 0))?;

		match task.plugin.prio {
			Priority::Low => self.macro_.send(PreloadOp::Rule(task).into(), NORMAL).await?,
			Priority::Normal => self.macro_.send(PreloadOp::Rule(task).into(), HIGH).await?,
			Priority::High => self.work(PreloadOp::Rule(task)).await?,
		}
		self.succ(id)
	}

	pub async fn size(&self, task: PreloadOpSize) -> Result<()> {
		let id = task.id;

		self.prog.send(TaskProg::New(id, 0))?;
		self.work(PreloadOp::Size(task)).await?;
		self.succ(id)
	}
}

impl Preload {
	#[inline]
	fn succ(&self, id: usize) -> Result<()> { Ok(self.prog.send(TaskProg::Succ(id))?) }

	#[inline]
	fn fail(&self, id: usize, reason: String) -> Result<()> {
		Ok(self.prog.send(TaskProg::Fail(id, reason))?)
	}
}
