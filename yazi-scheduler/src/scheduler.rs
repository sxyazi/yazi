use std::{borrow::Cow, ffi::OsString, sync::Arc, time::Duration};

use futures::{future::BoxFuture, FutureExt};
use parking_lot::Mutex;
use tokio::{fs, select, sync::{mpsc::{self, UnboundedReceiver}, oneshot}, task::JoinHandle};
use yazi_config::{open::Opener, plugin::PluginRule, TASKS};
use yazi_dds::ValueSendable;
use yazi_shared::{fs::{unique_path, Url}, Throttle};

use super::{Ongoing, TaskProg, TaskStage};
use crate::{file::{File, FileOpDelete, FileOpLink, FileOpPaste, FileOpTrash}, plugin::{Plugin, PluginOpEntry}, preload::{Preload, PreloadOpRule, PreloadOpSize}, process::{Process, ProcessOpBg, ProcessOpBlock, ProcessOpOrphan}, TaskKind, TaskOp, HIGH, LOW, NORMAL};

pub struct Scheduler {
	pub file:    Arc<File>,
	pub plugin:  Arc<Plugin>,
	pub preload: Arc<Preload>,
	pub process: Arc<Process>,

	micro:       async_priority_channel::Sender<BoxFuture<'static, ()>, u8>,
	prog:        mpsc::UnboundedSender<TaskProg>,
	handles:     Vec<JoinHandle<()>>,
	pub ongoing: Arc<Mutex<Ongoing>>,
}

impl Scheduler {
	pub fn serve() -> Self {
		let (micro_tx, micro_rx) = async_priority_channel::unbounded();
		let (macro_tx, macro_rx) = async_priority_channel::unbounded();
		let (prog_tx, prog_rx) = mpsc::unbounded_channel();

		let mut scheduler = Self {
			file:    Arc::new(File::new(macro_tx.clone(), prog_tx.clone())),
			plugin:  Arc::new(Plugin::new(macro_tx.clone(), prog_tx.clone())),
			preload: Arc::new(Preload::new(macro_tx.clone(), prog_tx.clone())),
			process: Arc::new(Process::new(prog_tx.clone())),

			micro:   micro_tx,
			prog:    prog_tx,
			handles: Vec::with_capacity(TASKS.micro_workers as usize + TASKS.macro_workers as usize + 1),
			ongoing: Default::default(),
		};

		for _ in 0..TASKS.micro_workers {
			scheduler.handles.push(scheduler.schedule_micro(micro_rx.clone()));
		}
		for _ in 0..TASKS.macro_workers {
			scheduler.handles.push(scheduler.schedule_macro(micro_rx.clone(), macro_rx.clone()));
		}
		scheduler.progress(prog_rx);
		scheduler
	}

	pub fn cancel(&self, id: usize) -> bool {
		let mut ongoing = self.ongoing.lock();

		if let Some(hook) = ongoing.hooks.remove(&id) {
			self.micro.try_send(hook(true), HIGH).ok();
			return false;
		}

		ongoing.all.remove(&id).is_some()
	}

	pub fn shutdown(&self) {
		for handle in &self.handles {
			handle.abort();
		}
	}

	pub fn file_cut(&self, from: Url, mut to: Url, force: bool) {
		let mut ongoing = self.ongoing.lock();
		let id = ongoing.add(TaskKind::User, format!("Cut {:?} to {:?}", from, to));

		ongoing.hooks.insert(id, {
			let from = from.clone();
			let ongoing = self.ongoing.clone();

			Box::new(move |canceled: bool| {
				async move {
					if !canceled {
						File::remove_empty_dirs(&from).await;
					}
					ongoing.lock().try_remove(id, TaskStage::Hooked);
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

	pub fn file_copy(&self, from: Url, mut to: Url, force: bool, follow: bool) {
		let name = format!("Copy {:?} to {:?}", from, to);
		let id = self.ongoing.lock().add(TaskKind::User, name);

		let file = self.file.clone();
		_ = self.micro.try_send(
			async move {
				if !force {
					to = unique_path(to).await;
				}
				file.paste(FileOpPaste { id, from, to, cut: false, follow, retry: 0 }).await.ok();
			}
			.boxed(),
			LOW,
		);
	}

	pub fn file_link(&self, from: Url, mut to: Url, relative: bool, force: bool) {
		let name = format!("Link {from:?} to {to:?}");
		let id = self.ongoing.lock().add(TaskKind::User, name);

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
		let mut ongoing = self.ongoing.lock();
		let id = ongoing.add(TaskKind::User, format!("Delete {:?}", target));

		ongoing.hooks.insert(id, {
			let target = target.clone();
			let ongoing = self.ongoing.clone();

			Box::new(move |canceled: bool| {
				async move {
					if !canceled {
						fs::remove_dir_all(target).await.ok();
					}
					ongoing.lock().try_remove(id, TaskStage::Hooked);
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
		let id = self.ongoing.lock().add(TaskKind::User, name);

		let file = self.file.clone();
		_ = self.micro.try_send(
			async move {
				file.trash(FileOpTrash { id, target, length: 0 }).await.ok();
			}
			.boxed(),
			LOW,
		);
	}

	pub fn plugin_micro(&self, name: String, args: Vec<ValueSendable>) {
		let id = self.ongoing.lock().add(TaskKind::User, format!("Run micro plugin `{name}`"));

		let plugin = self.plugin.clone();
		_ = self.micro.try_send(
			async move {
				plugin.micro(PluginOpEntry { id, name, args }).await.ok();
			}
			.boxed(),
			NORMAL,
		);
	}

	pub fn plugin_macro(&self, name: String, args: Vec<ValueSendable>) {
		let id = self.ongoing.lock().add(TaskKind::User, format!("Run macro plugin `{name}`"));

		self.plugin.macro_(PluginOpEntry { id, name, args }).ok();
	}

	pub fn preload_paged(&self, rule: &PluginRule, targets: Vec<&yazi_shared::fs::File>) {
		let id = self.ongoing.lock().add(
			TaskKind::Preload,
			format!("Run preloader `{}` with {} target(s)", rule.cmd.name, targets.len()),
		);

		let plugin = rule.into();
		let targets = targets.into_iter().cloned().collect();
		let preload = self.preload.clone();
		_ = self.micro.try_send(
			async move {
				preload.rule(PreloadOpRule { id, plugin, targets }).await.ok();
			}
			.boxed(),
			NORMAL,
		);
	}

	pub fn preload_size(&self, targets: Vec<&Url>) {
		let throttle = Arc::new(Throttle::new(targets.len(), Duration::from_millis(300)));
		let mut ongoing = self.ongoing.lock();

		for target in targets {
			let id = ongoing.add(TaskKind::Preload, format!("Calculate the size of {:?}", target));
			let target = target.clone();
			let throttle = throttle.clone();

			let preload = self.preload.clone();
			_ = self.micro.try_send(
				async move {
					preload.size(PreloadOpSize { id, target, throttle }).await.ok();
				}
				.boxed(),
				NORMAL,
			);
		}
	}

	pub fn process_open(
		&self,
		opener: Cow<'static, Opener>,
		args: Vec<OsString>,
		done: Option<oneshot::Sender<()>>,
	) {
		let name = {
			let args = args.iter().map(|a| a.to_string_lossy()).collect::<Vec<_>>().join(" ");
			if args.is_empty() {
				format!("Run {:?}", opener.run)
			} else {
				format!("Run {:?} with `{args}`", opener.run)
			}
		};

		let (cancel_tx, cancel_rx) = mpsc::channel(1);
		let mut ongoing = self.ongoing.lock();

		let id = ongoing.add(TaskKind::User, name);
		ongoing.hooks.insert(id, {
			let ongoing = self.ongoing.clone();
			Box::new(move |canceled: bool| {
				async move {
					if canceled {
						cancel_tx.send(()).await.ok();
						cancel_tx.closed().await;
					}
					if let Some(tx) = done {
						tx.send(()).ok();
					}
					ongoing.lock().try_remove(id, TaskStage::Hooked);
				}
				.boxed()
			})
		});

		let cmd = OsString::from(&opener.run);
		let process = self.process.clone();
		_ = self.micro.try_send(
			async move {
				if opener.block {
					process.block(ProcessOpBlock { id, cmd, args }).await.ok();
				} else if opener.orphan {
					process.orphan(ProcessOpOrphan { id, cmd, args }).await.ok();
				} else {
					process.bg(ProcessOpBg { id, cmd, args, cancel: cancel_rx }).await.ok();
				}
			}
			.boxed(),
			NORMAL,
		);
	}

	fn schedule_micro(
		&self,
		rx: async_priority_channel::Receiver<BoxFuture<'static, ()>, u8>,
	) -> JoinHandle<()> {
		tokio::spawn(async move {
			loop {
				if let Ok((fut, _)) = rx.recv().await {
					fut.await;
				}
			}
		})
	}

	fn schedule_macro(
		&self,
		micro: async_priority_channel::Receiver<BoxFuture<'static, ()>, u8>,
		macro_: async_priority_channel::Receiver<TaskOp, u8>,
	) -> JoinHandle<()> {
		let file = self.file.clone();
		let plugin = self.plugin.clone();
		let preload = self.preload.clone();

		let prog = self.prog.clone();
		let ongoing = self.ongoing.clone();

		tokio::spawn(async move {
			loop {
				select! {
					Ok((fut, _)) = micro.recv() => {
						fut.await;
					}
					Ok((op, _)) = macro_.recv() => {
						let id = op.id();
						if !ongoing.lock().exists(id) {
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
		})
	}

	fn progress(&self, mut rx: UnboundedReceiver<TaskProg>) -> JoinHandle<()> {
		let micro = self.micro.clone();
		let ongoing = self.ongoing.clone();

		tokio::spawn(async move {
			while let Some(op) = rx.recv().await {
				match op {
					TaskProg::New(id, size) => {
						if let Some(task) = ongoing.lock().get_mut(id) {
							task.total += 1;
							task.found += size;
						}
					}
					TaskProg::Adv(id, succ, processed) => {
						let mut ongoing = ongoing.lock();
						if let Some(task) = ongoing.get_mut(id) {
							task.succ += succ;
							task.processed += processed;
						}
						if succ > 0 {
							if let Some(fut) = ongoing.try_remove(id, TaskStage::Pending) {
								micro.try_send(fut, LOW).ok();
							}
						}
					}
					TaskProg::Succ(id) => {
						if let Some(fut) = ongoing.lock().try_remove(id, TaskStage::Dispatched) {
							micro.try_send(fut, LOW).ok();
						}
					}
					TaskProg::Fail(id, reason) => {
						if let Some(task) = ongoing.lock().get_mut(id) {
							task.fail += 1;
							task.logs.push_str(&reason);
							task.logs.push('\n');

							if let Some(logger) = &task.logger {
								logger.send(reason).ok();
							}
						}
					}
					TaskProg::Log(id, line) => {
						if let Some(task) = ongoing.lock().get_mut(id) {
							task.logs.push_str(&line);
							task.logs.push('\n');

							if let Some(logger) = &task.logger {
								logger.send(line).ok();
							}
						}
					}
				}
			}
		})
	}
}
