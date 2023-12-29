use std::{ffi::OsStr, sync::Arc, time::Duration};

use futures::{future::BoxFuture, FutureExt};
use parking_lot::RwLock;
use tokio::{fs, select, sync::{mpsc::{self, UnboundedReceiver}, oneshot}};
use yazi_config::{open::Opener, plugin::PluginRule, TASKS};
use yazi_shared::{emit, event::Exec, fs::{unique_path, Url}, Layer, Throttle};

use super::{Running, TaskProg, TaskStage};
use crate::{file::{File, FileOpDelete, FileOpLink, FileOpPaste, FileOpTrash}, plugin::{Plugin, PluginOpEntry}, preload::{Preload, PreloadOpRule, PreloadOpSize}, process::{Process, ProcessOpOpen}, TaskKind, TaskOp, HIGH, LOW, NORMAL};

pub struct Scheduler {
	pub file:    Arc<File>,
	pub plugin:  Arc<Plugin>,
	pub preload: Arc<Preload>,
	pub process: Arc<Process>,

	micro:       async_priority_channel::Sender<BoxFuture<'static, ()>, u8>,
	prog:        mpsc::UnboundedSender<TaskProg>,
	pub running: Arc<RwLock<Running>>,
}

impl Scheduler {
	pub fn start() -> Self {
		let (micro_tx, micro_rx) = async_priority_channel::unbounded();
		let (macro_tx, macro_rx) = async_priority_channel::unbounded();
		let (prog_tx, prog_rx) = mpsc::unbounded_channel();

		let scheduler = Self {
			file:    Arc::new(File::new(macro_tx.clone(), prog_tx.clone())),
			plugin:  Arc::new(Plugin::new(macro_tx.clone(), prog_tx.clone())),
			preload: Arc::new(Preload::new(macro_tx.clone(), prog_tx.clone())),
			process: Arc::new(Process::new(prog_tx.clone())),

			micro:   micro_tx,
			prog:    prog_tx,
			running: Default::default(),
		};

		for _ in 0..TASKS.micro_workers {
			scheduler.schedule_micro(micro_rx.clone());
		}
		for _ in 0..TASKS.macro_workers {
			scheduler.schedule_macro(micro_rx.clone(), macro_rx.clone());
		}
		scheduler.progress(prog_rx);
		scheduler
	}

	fn schedule_micro(&self, rx: async_priority_channel::Receiver<BoxFuture<'static, ()>, u8>) {
		tokio::spawn(async move {
			loop {
				if let Ok((fut, _)) = rx.recv().await {
					fut.await;
				}
			}
		});
	}

	fn schedule_macro(
		&self,
		micro: async_priority_channel::Receiver<BoxFuture<'static, ()>, u8>,
		macro_: async_priority_channel::Receiver<TaskOp, u8>,
	) {
		let file = self.file.clone();
		let plugin = self.plugin.clone();
		let preload = self.preload.clone();

		let prog = self.prog.clone();
		let running = self.running.clone();

		tokio::spawn(async move {
			loop {
				select! {
					Ok((fut, _)) = micro.recv() => {
						fut.await;
					}
					Ok((op, _)) = macro_.recv() => {
						let id = op.id();
						if !running.read().exists(id) {
							continue;
						}

						let result = match op {
							TaskOp::File(op) => file.work(*op).await,
							TaskOp::Plugin(op) => plugin.work(*op).await,
							TaskOp::Preload(op) => preload.work(*op).await,
						};

						if let Err(e) = result {
							prog.send(TaskProg::Fail(id, format!("Failed to work on this task: {:?}", e))).ok();
						}
					}
				}
			}
		});
	}

	fn progress(&self, mut rx: UnboundedReceiver<TaskProg>) {
		let micro = self.micro.clone();
		let running = self.running.clone();

		tokio::spawn(async move {
			while let Some(op) = rx.recv().await {
				match op {
					TaskProg::New(id, size) => {
						if let Some(task) = running.write().get_mut(id) {
							task.total += 1;
							task.found += size;
						}
					}
					TaskProg::Adv(id, succ, processed) => {
						let mut running = running.write();
						if let Some(task) = running.get_mut(id) {
							task.succ += succ;
							task.processed += processed;
						}
						if succ > 0 {
							if let Some(fut) = running.try_remove(id, TaskStage::Pending) {
								micro.try_send(fut, NORMAL).ok();
							}
						}
					}
					TaskProg::Succ(id) => {
						if let Some(fut) = running.write().try_remove(id, TaskStage::Dispatched) {
							micro.try_send(fut, NORMAL).ok();
						}
					}
					TaskProg::Fail(id, reason) => {
						if let Some(task) = running.write().get_mut(id) {
							task.fail += 1;
							task.logs.push_str(&reason);
							task.logs.push('\n');

							if let Some(logger) = &task.logger {
								logger.send(reason).ok();
							}
						}
					}
					TaskProg::Log(id, line) => {
						if let Some(task) = running.write().get_mut(id) {
							task.logs.push_str(&line);
							task.logs.push('\n');

							if let Some(logger) = &task.logger {
								logger.send(line).ok();
							}
						}
					}
				}
			}
		});
	}

	pub fn cancel(&self, id: usize) -> bool {
		let mut running = self.running.write();
		let b = running.all.remove(&id).is_some();

		if let Some(hook) = running.hooks.remove(&id) {
			self.micro.try_send(hook(true), HIGH).ok();
		}
		b
	}

	pub async fn app_stop() {
		let (tx, rx) = oneshot::channel::<()>();
		emit!(Call(Exec::call("stop", vec!["true".to_string()]).with_data(Some(tx)).vec(), Layer::App));
		rx.await.ok();
	}

	pub fn app_resume() {
		emit!(Call(
			Exec::call("stop", vec!["false".to_string()]).with_data(None::<oneshot::Sender<()>>).vec(),
			Layer::App
		));
	}

	pub fn file_cut(&self, from: Url, mut to: Url, force: bool) {
		let mut running = self.running.write();
		let id = running.add(TaskKind::User, format!("Cut {:?} to {:?}", from, to));

		running.hooks.insert(id, {
			let from = from.clone();
			let running = self.running.clone();

			Box::new(move |canceled: bool| {
				async move {
					if !canceled {
						File::remove_empty_dirs(&from).await;
					}
					running.write().try_remove(id, TaskStage::Hooked);
				}
				.boxed()
			})
		});

		let file = self.file.clone();
		_ = self.micro.try_send(
			async move {
				if !force {
					to = unique_path(to).await;
				}
				file.paste(FileOpPaste { id, from, to, cut: true, follow: false, retry: 0 }).await.ok();
			}
			.boxed(),
			LOW,
		);
	}

	pub fn file_copy(&self, from: Url, mut to: Url, force: bool) {
		let name = format!("Copy {:?} to {:?}", from, to);
		let id = self.running.write().add(TaskKind::User, name);

		let file = self.file.clone();
		_ = self.micro.try_send(
			async move {
				if !force {
					to = unique_path(to).await;
				}
				file.paste(FileOpPaste { id, from, to, cut: false, follow: true, retry: 0 }).await.ok();
			}
			.boxed(),
			LOW,
		);
	}

	pub fn file_link(&self, from: Url, mut to: Url, relative: bool, force: bool) {
		let name = format!("Link {from:?} to {to:?}");
		let id = self.running.write().add(TaskKind::User, name);

		let file = self.file.clone();
		_ = self.micro.try_send(
			async move {
				if !force {
					to = unique_path(to).await;
				}
				file
					.link(FileOpLink { id, from, to, meta: None, resolve: false, relative, delete: false })
					.await
					.ok();
			}
			.boxed(),
			LOW,
		);
	}

	pub fn file_delete(&self, target: Url) {
		let mut running = self.running.write();
		let id = running.add(TaskKind::User, format!("Delete {:?}", target));

		running.hooks.insert(id, {
			let target = target.clone();
			let running = self.running.clone();

			Box::new(move |canceled: bool| {
				async move {
					if !canceled {
						fs::remove_dir_all(target).await.ok();
					}
					running.write().try_remove(id, TaskStage::Hooked);
				}
				.boxed()
			})
		});

		let file = self.file.clone();
		_ = self.micro.try_send(
			async move {
				file.delete(FileOpDelete { id, target, length: 0 }).await.ok();
			}
			.boxed(),
			LOW,
		);
	}

	pub fn file_trash(&self, target: Url) {
		let name = format!("Trash {:?}", target);
		let id = self.running.write().add(TaskKind::User, name);

		let file = self.file.clone();
		_ = self.micro.try_send(
			async move {
				file.trash(FileOpTrash { id, target, length: 0 }).await.ok();
			}
			.boxed(),
			LOW,
		);
	}

	pub fn plugin_micro(&self, name: String) {
		let id = self.running.write().add(TaskKind::User, format!("Run micro plugin `{name}`"));

		let plugin = self.plugin.clone();
		_ = self.micro.try_send(
			async move {
				plugin.micro(PluginOpEntry { id, name }).await.ok();
			}
			.boxed(),
			HIGH,
		);
	}

	pub fn plugin_macro(&self, name: String) {
		let id = self.running.write().add(TaskKind::User, format!("Run macro plugin `{name}`"));

		self.plugin.macro_(PluginOpEntry { id, name }).ok();
	}

	pub fn preload_paged(&self, rule: &PluginRule, targets: Vec<&yazi_shared::fs::File>) {
		let id = self.running.write().add(
			TaskKind::Preload,
			format!("Run preloader `{}` with {} target(s)", rule.exec.cmd, targets.len()),
		);

		let (rule_id, rule_multi) = (rule.id, rule.multi);
		let cmd = rule.exec.cmd.clone();
		let targets = targets.into_iter().cloned().collect();

		let preload = self.preload.clone();
		_ = self.micro.try_send(
			async move {
				preload.rule(PreloadOpRule { id, rule_id, rule_multi, plugin: cmd, targets }).await.ok();
			}
			.boxed(),
			HIGH,
		);
	}

	pub fn preload_size(&self, targets: Vec<&Url>) {
		let throttle = Arc::new(Throttle::new(targets.len(), Duration::from_millis(300)));
		let mut running = self.running.write();
		for target in targets {
			let id = running.add(TaskKind::Preload, format!("Calculate the size of {:?}", target));
			let target = target.clone();
			let throttle = throttle.clone();

			let preload = self.preload.clone();
			_ = self.micro.try_send(
				async move {
					preload.size(PreloadOpSize { id, target, throttle }).await.ok();
				}
				.boxed(),
				HIGH,
			);
		}
	}

	pub fn process_open(&self, opener: &Opener, args: &[impl AsRef<OsStr>]) {
		let name = {
			let s = format!("Execute `{}`", opener.exec);
			let args = args.iter().map(|a| a.as_ref().to_string_lossy()).collect::<Vec<_>>().join(" ");
			if args.is_empty() { s } else { format!("{s} with `{args}`") }
		};

		let mut running = self.running.write();
		let id = running.add(TaskKind::User, name);

		let (cancel_tx, mut cancel_rx) = oneshot::channel();
		running.hooks.insert(id, {
			let running = self.running.clone();
			Box::new(move |canceled: bool| {
				async move {
					if canceled {
						cancel_rx.close();
					}
					running.write().try_remove(id, TaskStage::Hooked);
				}
				.boxed()
			})
		});

		let args = args.iter().map(|a| a.as_ref().to_os_string()).collect::<Vec<_>>();
		tokio::spawn({
			let process = self.process.clone();
			let opener = opener.clone();
			async move {
				process
					.open(ProcessOpOpen {
						id,
						cmd: opener.exec.into(),
						args,
						block: opener.block,
						orphan: opener.orphan,
						cancel: cancel_tx,
					})
					.await
					.ok();
			}
		});
	}
}
