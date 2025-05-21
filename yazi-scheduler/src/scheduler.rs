use std::{ffi::OsString, future::Future, sync::Arc, time::Duration};

use anyhow::Result;
use futures::{FutureExt, future::BoxFuture};
use parking_lot::Mutex;
use tokio::{fs, select, sync::mpsc::{self, UnboundedReceiver}, task::JoinHandle};
use yazi_config::{YAZI, plugin::{Fetcher, Preloader}};
use yazi_dds::Pump;
use yazi_fs::{must_be_dir, remove_dir_clean, unique_name};
use yazi_proxy::{MgrProxy, options::{PluginOpt, ProcessExecOpt}};
use yazi_shared::{Throttle, url::Url};

use super::{Ongoing, TaskProg, TaskStage};
use crate::{HIGH, LOW, NORMAL, TaskKind, TaskOp, file::{File, FileInDelete, FileInHardlink, FileInLink, FileInPaste, FileInTrash}, plugin::{Plugin, PluginInEntry}, prework::{Prework, PreworkInFetch, PreworkInLoad, PreworkInSize}, process::{Process, ProcessInBg, ProcessInBlock, ProcessInOrphan}};

pub struct Scheduler {
	pub file:    Arc<File>,
	pub plugin:  Arc<Plugin>,
	pub prework: Arc<Prework>,
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
			prework: Arc::new(Prework::new(macro_tx.clone(), prog_tx.clone())),
			process: Arc::new(Process::new(prog_tx.clone())),

			micro:   micro_tx,
			prog:    prog_tx,
			handles: Vec::with_capacity(
				YAZI.tasks.micro_workers as usize + YAZI.tasks.macro_workers as usize + 1,
			),
			ongoing: Default::default(),
		};

		for _ in 0..YAZI.tasks.micro_workers {
			scheduler.handles.push(scheduler.schedule_micro(micro_rx.clone()));
		}
		for _ in 0..YAZI.tasks.macro_workers {
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
		let id = ongoing.add(TaskKind::User, format!("Cut {from} to {to}"));

		if to.starts_with(&from) && to != from {
			self.new_and_fail(id, "Cannot cut directory into itself").ok();
			return;
		}

		ongoing.hooks.insert(id, {
			let ongoing = self.ongoing.clone();
			let (from, to) = (from.clone(), to.clone());

			Box::new(move |canceled: bool| {
				async move {
					if !canceled {
						remove_dir_clean(&from).await;
						Pump::push_move(from, to);
					}
					ongoing.lock().try_remove(id, TaskStage::Hooked);
				}
				.boxed()
			})
		});

		let file = self.file.clone();
		self.send_micro(id, LOW, async move {
			if !force {
				to = unique_name(to, must_be_dir(&from)).await?;
			}
			file.paste(FileInPaste { id, from, to, cha: None, cut: true, follow: false, retry: 0 }).await
		});
	}

	pub fn file_copy(&self, from: Url, mut to: Url, force: bool, follow: bool) {
		let id = self.ongoing.lock().add(TaskKind::User, format!("Copy {from} to {to}"));

		if to.starts_with(&from) && to != from {
			self.new_and_fail(id, "Cannot copy directory into itself").ok();
			return;
		}

		let file = self.file.clone();
		self.send_micro(id, LOW, async move {
			if !force {
				to = unique_name(to, must_be_dir(&from)).await?;
			}
			file.paste(FileInPaste { id, from, to, cha: None, cut: false, follow, retry: 0 }).await
		});
	}

	pub fn file_link(&self, from: Url, mut to: Url, relative: bool, force: bool) {
		let id = self.ongoing.lock().add(TaskKind::User, format!("Link {from} to {to}"));

		let file = self.file.clone();
		self.send_micro(id, LOW, async move {
			if !force {
				to = unique_name(to, must_be_dir(&from)).await?;
			}
			file
				.link(FileInLink { id, from, to, cha: None, resolve: false, relative, delete: false })
				.await
		});
	}

	pub fn file_hardlink(&self, from: Url, mut to: Url, force: bool, follow: bool) {
		let id = self.ongoing.lock().add(TaskKind::User, format!("Hardlink {from} to {to}"));

		if to.starts_with(&from) && to != from {
			self.new_and_fail(id, "Cannot hardlink directory into itself").ok();
			return;
		}

		let file = self.file.clone();
		self.send_micro(id, LOW, async move {
			if !force {
				to = unique_name(to, must_be_dir(&from)).await?;
			}
			file.hardlink(FileInHardlink { id, from, to, cha: None, follow }).await
		});
	}

	pub fn file_delete(&self, target: Url) {
		let mut ongoing = self.ongoing.lock();
		let id = ongoing.add(TaskKind::User, format!("Delete {target}"));

		ongoing.hooks.insert(id, {
			let target = target.clone();
			let ongoing = self.ongoing.clone();

			Box::new(move |canceled: bool| {
				async move {
					if !canceled {
						fs::remove_dir_all(&target).await.ok();
						MgrProxy::update_tasks(&target);
						Pump::push_delete(target);
					}
					ongoing.lock().try_remove(id, TaskStage::Hooked);
				}
				.boxed()
			})
		});

		let file = self.file.clone();
		self.send_micro(
			id,
			LOW,
			async move { file.delete(FileInDelete { id, target, length: 0 }).await },
		);
	}

	pub fn file_trash(&self, target: Url) {
		let mut ongoing = self.ongoing.lock();
		let id = ongoing.add(TaskKind::User, format!("Trash {target}"));

		ongoing.hooks.insert(id, {
			let target = target.clone();
			let ongoing = self.ongoing.clone();

			Box::new(move |canceled: bool| {
				async move {
					if !canceled {
						MgrProxy::update_tasks(&target);
						Pump::push_trash(target);
					}
					ongoing.lock().try_remove(id, TaskStage::Hooked);
				}
				.boxed()
			})
		});

		let file = self.file.clone();
		self.send_micro(id, LOW, async move {
			file.trash(FileInTrash { id, target: target.clone(), length: 0 }).await
		})
	}

	pub fn plugin_micro(&self, opt: PluginOpt) {
		let id = self.ongoing.lock().add(TaskKind::User, format!("Run micro plugin `{}`", opt.id));

		let plugin = self.plugin.clone();
		self.send_micro(id, NORMAL, async move { plugin.micro(PluginInEntry { id, opt }).await });
	}

	pub fn plugin_macro(&self, opt: PluginOpt) {
		let id = self.ongoing.lock().add(TaskKind::User, format!("Run macro plugin `{}`", opt.id));

		self.plugin.r#macro(PluginInEntry { id, opt }).ok();
	}

	pub fn fetch_paged(&self, fetcher: &'static Fetcher, targets: Vec<yazi_fs::File>) {
		let id = self.ongoing.lock().add(
			TaskKind::Preload,
			format!("Run fetcher `{}` with {} target(s)", fetcher.run.name, targets.len()),
		);

		let prework = self.prework.clone();
		self.send_micro(id, NORMAL, async move {
			prework.fetch(PreworkInFetch { id, plugin: fetcher, targets }).await
		});
	}

	pub fn preload_paged(&self, preloader: &'static Preloader, target: &yazi_fs::File) {
		let id =
			self.ongoing.lock().add(TaskKind::Preload, format!("Run preloader `{}`", preloader.run.name));

		let target = target.clone();
		let prework = self.prework.clone();
		self.send_micro(id, NORMAL, async move {
			prework.load(PreworkInLoad { id, plugin: preloader, target }).await
		});
	}

	pub fn prework_size(&self, targets: Vec<&Url>) {
		let throttle = Arc::new(Throttle::new(targets.len(), Duration::from_millis(300)));
		let mut ongoing = self.ongoing.lock();

		for target in targets {
			let id = ongoing.add(TaskKind::Preload, format!("Calculate the size of {target}"));
			let target = target.clone();
			let throttle = throttle.clone();

			let prework = self.prework.clone();
			self.send_micro(id, NORMAL, async move {
				prework.size(PreworkInSize { id, target, throttle }).await
			});
		}
	}

	pub fn process_open(&self, ProcessExecOpt { cwd, opener, args, done }: ProcessExecOpt) {
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
		self.send_micro(id, NORMAL, async move {
			if opener.block {
				process.block(ProcessInBlock { id, cwd, cmd, args }).await
			} else if opener.orphan {
				process.orphan(ProcessInOrphan { id, cwd, cmd, args }).await
			} else {
				process.bg(ProcessInBg { id, cwd, cmd, args, cancel: cancel_rx }).await
			}
		});
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
		r#macro: async_priority_channel::Receiver<TaskOp, u8>,
	) -> JoinHandle<()> {
		let file = self.file.clone();
		let plugin = self.plugin.clone();
		let prework = self.prework.clone();

		let prog = self.prog.clone();
		let ongoing = self.ongoing.clone();

		tokio::spawn(async move {
			loop {
				select! {
					Ok((fut, _)) = micro.recv() => {
						fut.await;
					}
					Ok((r#in, _)) = r#macro.recv() => {
						let id = r#in.id();
						if !ongoing.lock().exists(id) {
							continue;
						}

						let result = match r#in {
							TaskOp::File(r#in) => file.work(*r#in).await,
							TaskOp::Plugin(r#in) => plugin.work(*r#in).await,
							TaskOp::Prework(r#in) => prework.work(*r#in).await,
						};

						if let Err(e) = result {
							prog.send(TaskProg::Fail(id, format!("Failed to work on this task: {e:?}"))).ok();
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
			while let Some(r#in) = rx.recv().await {
				match r#in {
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

	fn send_micro<F>(&self, id: usize, priority: u8, f: F)
	where
		F: Future<Output = Result<()>> + Send + 'static,
	{
		let prog = self.prog.clone();
		_ = self.micro.try_send(
			async move {
				if let Err(e) = f.await {
					prog.send(TaskProg::New(id, 0)).ok();
					prog.send(TaskProg::Fail(id, format!("Task initialization failed:\n{e:?}"))).ok();
				}
			}
			.boxed(),
			priority,
		);
	}

	fn new_and_fail(&self, id: usize, reason: &str) -> Result<()> {
		self.prog.send(TaskProg::New(id, 0))?;
		self.prog.send(TaskProg::Fail(id, reason.to_owned()))?;
		Ok(())
	}
}
