use std::{ffi::OsStr, sync::Arc, time::Duration};

use futures::{future::BoxFuture, FutureExt};
use parking_lot::RwLock;
use tokio::{fs, select, sync::{mpsc::{self, UnboundedReceiver}, oneshot}};
use yazi_config::{open::Opener, TASKS};
use yazi_shared::{emit, event::Exec, fs::{unique_path, Url}, Layer, Throttle};

use super::{Running, TaskOp, TaskStage};
use crate::{workers::{File, FileOpDelete, FileOpLink, FileOpPaste, FileOpTrash, Precache, PrecacheOpMime, PrecacheOpSize, Process, ProcessOpOpen}, TaskKind};

pub struct Scheduler {
	file:     Arc<File>,
	precache: Arc<Precache>,
	process:  Arc<Process>,

	todo:        async_channel::Sender<BoxFuture<'static, ()>>,
	prog:        mpsc::UnboundedSender<TaskOp>,
	pub running: Arc<RwLock<Running>>,
}

impl Scheduler {
	pub fn start() -> Self {
		let (todo_tx, todo_rx) = async_channel::unbounded();
		let (prog_tx, prog_rx) = mpsc::unbounded_channel();

		let scheduler = Self {
			file:     Arc::new(File::new(prog_tx.clone())),
			precache: Arc::new(Precache::new(prog_tx.clone())),
			process:  Arc::new(Process::new(prog_tx.clone())),

			todo:    todo_tx,
			prog:    prog_tx,
			running: Default::default(),
		};

		for _ in 0..TASKS.micro_workers {
			scheduler.schedule_micro(todo_rx.clone());
		}
		for _ in 0..TASKS.macro_workers {
			scheduler.schedule_macro(todo_rx.clone());
		}
		scheduler.progress(prog_rx);
		scheduler
	}

	fn schedule_micro(&self, rx: async_channel::Receiver<BoxFuture<'static, ()>>) {
		tokio::spawn(async move {
			loop {
				if let Ok(fut) = rx.recv().await {
					fut.await;
				}
			}
		});
	}

	fn schedule_macro(&self, rx: async_channel::Receiver<BoxFuture<'static, ()>>) {
		let file = self.file.clone();
		let precache = self.precache.clone();

		let prog = self.prog.clone();
		let running = self.running.clone();

		tokio::spawn(async move {
			loop {
				if let Ok(fut) = rx.try_recv() {
					fut.await;
					continue;
				}

				select! {
					Ok(fut) = rx.recv() => {
						fut.await;
					}
					Ok((id, mut op)) = file.recv() => {
						if !running.read().exists(id) {
							continue;
						}
						if let Err(e) = file.work(&mut op).await {
							prog.send(TaskOp::Fail(id, format!("Failed to work on this task: {:?}", e))).ok();
						}
					}
					Ok((id, mut op)) = precache.recv() => {
						if !running.read().exists(id) {
							continue;
						}
						if let Err(e) = precache.work(&mut op).await {
							prog.send(TaskOp::Fail(id, format!("Failed to work on this task: {:?}", e))).ok();
						}
					}
				}
			}
		});
	}

	fn progress(&self, mut rx: UnboundedReceiver<TaskOp>) {
		let todo = self.todo.clone();
		let running = self.running.clone();

		tokio::spawn(async move {
			while let Some(op) = rx.recv().await {
				match op {
					TaskOp::New(id, size) => {
						if let Some(task) = running.write().get_mut(id) {
							task.total += 1;
							task.found += size;
						}
					}
					TaskOp::Adv(id, succ, processed) => {
						let mut running = running.write();
						if let Some(task) = running.get_mut(id) {
							task.succ += succ;
							task.processed += processed;
						}
						if succ > 0 {
							if let Some(fut) = running.try_remove(id, TaskStage::Pending) {
								todo.send_blocking(fut).ok();
							}
						}
					}
					TaskOp::Succ(id) => {
						if let Some(fut) = running.write().try_remove(id, TaskStage::Dispatched) {
							todo.send_blocking(fut).ok();
						}
					}
					TaskOp::Fail(id, reason) => {
						if let Some(task) = running.write().get_mut(id) {
							task.fail += 1;
							task.logs.push_str(&reason);
							task.logs.push('\n');

							if let Some(logger) = &task.logger {
								logger.send(reason).ok();
							}
						}
					}
					TaskOp::Log(id, line) => {
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
			self.todo.send_blocking(hook(true)).ok();
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
		let id = running.add(format!("Cut {:?} to {:?}", from, to));

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

		_ = self.todo.send_blocking({
			let file = self.file.clone();
			async move {
				if !force {
					to = unique_path(to).await;
				}
				file.paste(FileOpPaste { id, from, to, cut: true, follow: false, retry: 0 }).await.ok();
			}
			.boxed()
		});
	}

	pub fn file_copy(&self, from: Url, mut to: Url, force: bool) {
		let name = format!("Copy {:?} to {:?}", from, to);
		let id = self.running.write().add(name);

		_ = self.todo.send_blocking({
			let file = self.file.clone();
			async move {
				if !force {
					to = unique_path(to).await;
				}
				file.paste(FileOpPaste { id, from, to, cut: false, follow: true, retry: 0 }).await.ok();
			}
			.boxed()
		});
	}

	pub fn file_link(&self, from: Url, mut to: Url, relative: bool, force: bool) {
		let name = format!("Link {from:?} to {to:?}");
		let id = self.running.write().add(name);

		_ = self.todo.send_blocking({
			let file = self.file.clone();
			async move {
				if !force {
					to = unique_path(to).await;
				}
				file
					.link(FileOpLink { id, from, to, meta: None, resolve: false, relative, delete: false })
					.await
					.ok();
			}
			.boxed()
		});
	}

	pub fn file_delete(&self, target: Url) {
		let mut running = self.running.write();
		let id = running.add(format!("Delete {:?}", target));

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

		_ = self.todo.send_blocking({
			let file = self.file.clone();
			async move {
				file.delete(FileOpDelete { id, target, length: 0 }).await.ok();
			}
			.boxed()
		});
	}

	pub fn file_trash(&self, target: Url) {
		let name = format!("Trash {:?}", target);
		let id = self.running.write().add(name);

		_ = self.todo.send_blocking({
			let file = self.file.clone();
			async move {
				file.trash(FileOpTrash { id, target, length: 0 }).await.ok();
			}
			.boxed()
		});
	}

	pub fn process_open(&self, opener: &Opener, args: &[impl AsRef<OsStr>]) {
		let name = {
			let s = format!("Execute `{}`", opener.exec);
			let args = args.iter().map(|a| a.as_ref().to_string_lossy()).collect::<Vec<_>>().join(" ");
			if args.is_empty() { s } else { format!("{} with `{}`", s, args) }
		};

		let mut running = self.running.write();
		let id = running.add(name);

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

	pub fn precache_size(&self, targets: Vec<&Url>) {
		let throttle = Arc::new(Throttle::new(targets.len(), Duration::from_millis(300)));
		let mut handing = self.precache.size_handing.lock();
		let mut running = self.running.write();

		for target in targets {
			if !handing.contains(target) {
				handing.insert(target.clone());
			} else {
				continue;
			}

			let id = running.add(format!("Calculate the size of {:?}", target));
			if let Some(task) = self.running.clone().write().get_mut(id) {
				task.kind = TaskKind::PreCache;
			}

			_ = self.todo.send_blocking({
				let precache = self.precache.clone();
				let target = target.clone();
				let throttle = throttle.clone();
				async move {
					precache.size(PrecacheOpSize { id, target, throttle }).await.ok();
				}
				.boxed()
			});
		}
	}

	pub fn precache_mime(&self, targets: Vec<Url>) {
		let name = format!("Preload mimetype for {} files", targets.len());
		let id = self.running.write().add(name);

		if let Some(task) = self.running.clone().write().get_mut(id) {
			task.kind = TaskKind::PreCache;
		}

		_ = self.todo.send_blocking({
			let precache = self.precache.clone();
			async move {
				precache.mime(PrecacheOpMime { id, targets }).await.ok();
			}
			.boxed()
		});
	}

	pub fn precache_image(&self, targets: Vec<Url>) {
		let name = format!("Precache of {} image files", targets.len());
		let id = self.running.write().add(name);

		if let Some(task) = self.running.clone().write().get_mut(id) {
			task.kind = TaskKind::PreCache;
		}

		self.precache.image(id, targets).ok();
	}

	pub fn precache_video(&self, targets: Vec<Url>) {
		let name = format!("Precache of {} video files", targets.len());
		let id = self.running.write().add(name);

		if let Some(task) = self.running.clone().write().get_mut(id) {
			task.kind = TaskKind::PreCache;
		}

		self.precache.video(id, targets).ok();
	}

	pub fn precache_pdf(&self, targets: Vec<Url>) {
		let name = format!("Precache of {} PDF files", targets.len());
		let id = self.running.write().add(name);

		if let Some(task) = self.running.clone().write().get_mut(id) {
			task.kind = TaskKind::PreCache;
		}

		self.precache.pdf(id, targets).ok();
	}
}
