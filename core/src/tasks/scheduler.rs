use std::{ffi::{OsStr, OsString}, path::PathBuf, sync::Arc, time::Duration};

use async_channel::{Receiver, Sender};
use config::open::Opener;
use futures::{future::BoxFuture, FutureExt};
use parking_lot::RwLock;
use shared::{unique_path, Throttle};
use tokio::{fs, select, sync::{mpsc::{self, UnboundedReceiver}, oneshot}, time::sleep};
use tracing::{info, trace};

use super::{workers::{File, FileOpDelete, FileOpPaste, FileOpTrash, Precache, PrecacheOpMime, PrecacheOpSize, Process, ProcessOpOpen}, Running, TaskOp, TaskStage};
use crate::emit;

pub struct Scheduler {
	file:     Arc<File>,
	precache: Arc<Precache>,
	process:  Arc<Process>,

	todo:               Sender<BoxFuture<'static, ()>>,
	pub(super) running: Arc<RwLock<Running>>,
}

impl Scheduler {
	pub(super) fn start() -> Self {
		let (todo_tx, todo_rx) = async_channel::unbounded();
		let (prog_tx, prog_rx) = mpsc::unbounded_channel();

		let scheduler = Self {
			file:     Arc::new(File::new(prog_tx.clone())),
			precache: Arc::new(Precache::new(prog_tx.clone())),
			process:  Arc::new(Process::new(prog_tx)),

			todo:    todo_tx,
			running: Default::default(),
		};

		for _ in 0..5 {
			scheduler.schedule_micro(todo_rx.clone());
		}
		for _ in 0..5 {
			scheduler.schedule_macro(todo_rx.clone());
		}
		scheduler.progress(prog_rx);
		scheduler
	}

	fn schedule_micro(&self, rx: Receiver<BoxFuture<'static, ()>>) {
		tokio::spawn(async move {
			loop {
				if let Ok(fut) = rx.recv().await {
					fut.await;
				}
			}
		});
	}

	fn schedule_macro(&self, rx: Receiver<BoxFuture<'static, ()>>) {
		let file = self.file.clone();
		let precache = self.precache.clone();
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
							trace!("Skipping task {:?} as it was removed", op);
							continue;
						}
						if let Err(e) = file.work(&mut op).await {
							info!("Failed to work on task {:?}: {}", op, e);
						} else {
							trace!("Finished task {:?}", op);
						}
					}
					Ok((id, mut op)) = precache.recv() => {
						if !running.read().exists(id) {
							trace!("Skipping task {:?} as it was removed", op);
							continue;
						}
						if let Err(e) = precache.work(&mut op).await {
							info!("Failed to work on task {:?}: {}", op, e);
						} else {
							trace!("Finished task {:?}", op);
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
							task.found += 1;
							task.todo += size;
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
					TaskOp::Adv(id, processed, size) => {
						let mut running = running.write();
						if let Some(task) = running.get_mut(id) {
							task.processed += processed;
							task.done += size;
						}
						if processed > 0 {
							if let Some(fut) = running.try_remove(id, TaskStage::Pending) {
								todo.send_blocking(fut).ok();
							}
						}
					}
					TaskOp::Done(id) => {
						if let Some(fut) = running.write().try_remove(id, TaskStage::Dispatched) {
							todo.send_blocking(fut).ok();
						}
					}
				}
			}
		});

		let running = self.running.clone();
		let mut last = (100, 0);
		tokio::spawn(async move {
			loop {
				sleep(Duration::from_secs(1)).await;
				if running.read().is_empty() {
					if last != (100, 0) {
						last = (100, 0);
						emit!(Progress(last.0, last.1));
					}
					continue;
				}

				let mut tasks = 0u32;
				let mut left = 0;
				let mut progress = (0, 0);
				for task in running.read().values() {
					tasks += 1;
					left += task.found.saturating_sub(task.processed);
					progress = (progress.0 + task.done, progress.1 + task.todo);
				}

				let mut percent = match progress.1 {
					0 => 100u8,
					_ => 100.min(progress.0 * 100 / progress.1) as u8,
				};

				if tasks != 0 {
					percent = percent.min(99);
					left = left.max(1);
				}

				if last != (percent, left) {
					last = (percent, left);
					emit!(Progress(last.0, last.1));
				}
			}
		});
	}

	pub(super) fn cancel(&self, id: usize) -> bool {
		let mut running = self.running.write();
		let b = running.all.remove(&id).is_some();

		if let Some(hook) = running.hooks.remove(&id) {
			self.todo.send_blocking(hook(true)).ok();
		}
		b
	}

	pub(super) fn file_cut(&self, from: PathBuf, mut to: PathBuf, force: bool) {
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

		let _ = self.todo.send_blocking({
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

	pub(super) fn file_copy(&self, from: PathBuf, mut to: PathBuf, force: bool, follow: bool) {
		let name = format!("Copy {:?} to {:?}", from, to);
		let id = self.running.write().add(name);

		let _ = self.todo.send_blocking({
			let file = self.file.clone();
			async move {
				if !force {
					to = unique_path(to).await;
				}
				file.paste(FileOpPaste { id, from, to, cut: false, follow, retry: 0 }).await.ok();
			}
			.boxed()
		});
	}

	pub(super) fn file_delete(&self, target: PathBuf) {
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

		let _ = self.todo.send_blocking({
			let file = self.file.clone();
			async move {
				file.delete(FileOpDelete { id, target, length: 0 }).await.ok();
			}
			.boxed()
		});
	}

	pub(super) fn file_trash(&self, target: PathBuf) {
		let name = format!("Trash {:?}", target);
		let id = self.running.write().add(name);

		let _ = self.todo.send_blocking({
			let file = self.file.clone();
			async move {
				file.trash(FileOpTrash { id, target, length: 0 }).await.ok();
			}
			.boxed()
		});
	}

	pub(super) fn process_open(&self, opener: &Opener, args: &[impl AsRef<OsStr>]) {
		let args: Vec<OsString> = opener
			.args
			.iter()
			.map_while(|a| {
				if !a.starts_with('$') {
					return Some(vec![a.into()]);
				}
				if a == "$*" {
					return Some(args.iter().map(Into::into).collect());
				}
				a[1..].parse().ok().and_then(|n: usize| args.get(n)).map(|a| vec![a.into()])
			})
			.flatten()
			.collect();

		let mut running = self.running.write();
		let name = format!("Exec `{} {}`", opener.cmd, args.join(" ".as_ref()).to_string_lossy());
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

		tokio::spawn({
			let process = self.process.clone();
			let opener = opener.clone();
			async move {
				process
					.open(ProcessOpOpen { id, cmd: opener.cmd, args, block: opener.block, cancel: cancel_tx })
					.await
					.ok();
			}
		});
	}

	pub(super) fn precache_size(&self, targets: Vec<PathBuf>) {
		let throttle = Arc::new(Throttle::new(targets.len(), Duration::from_millis(300)));
		let mut handing = self.precache.size_handing.lock();
		let mut running = self.running.write();

		for target in targets {
			if !handing.contains(&target) {
				handing.insert(target.clone());
			} else {
				continue;
			}

			let id = running.add(format!("Calculate the size of {:?}", target));
			let _ = self.todo.send_blocking({
				let precache = self.precache.clone();
				let throttle = throttle.clone();
				async move {
					precache.size(PrecacheOpSize { id, target, throttle }).await.ok();
				}
				.boxed()
			});
		}
	}

	pub(super) fn precache_mime(&self, targets: Vec<PathBuf>) {
		let name = format!("Preload mimetype for {} files", targets.len());
		let id = self.running.write().add(name);

		let _ = self.todo.send_blocking({
			let precache = self.precache.clone();
			async move {
				precache.mime(PrecacheOpMime { id, targets }).await.ok();
			}
			.boxed()
		});
	}

	pub(super) fn precache_image(&self, targets: Vec<PathBuf>) {
		let name = format!("Precache of {} image files", targets.len());
		let id = self.running.write().add(name);

		self.precache.image(id, targets).ok();
	}

	pub(super) fn precache_video(&self, targets: Vec<PathBuf>) {
		let name = format!("Precache of {} video files", targets.len());
		let id = self.running.write().add(name);

		self.precache.video(id, targets).ok();
	}
}
