use std::{sync::Arc, time::Duration};

use parking_lot::Mutex;
use tokio::{select, sync::mpsc::{self, UnboundedReceiver}, task::JoinHandle};
use yazi_config::{YAZI, plugin::{Fetcher, Preloader}};
use yazi_parser::{app::PluginOpt, tasks::ProcessOpenOpt};
use yazi_shared::{CompletionToken, Id, Throttle, url::{UrlBuf, UrlLike}};

use super::{Ongoing, TaskOp};
use crate::{HIGH, LOW, NORMAL, Runner, TaskIn, TaskOps, file::{File, FileInCopy, FileInCut, FileInDelete, FileInDownload, FileInHardlink, FileInLink, FileInTrash, FileInUpload, FileOutCopy, FileOutCut, FileOutDownload, FileOutHardlink, FileOutUpload, FileProgCopy, FileProgCut, FileProgDelete, FileProgDownload, FileProgHardlink, FileProgLink, FileProgTrash, FileProgUpload}, hook::{Hook, HookInOutDelete, HookInOutDownload, HookInOutTrash}, plugin::{Plugin, PluginInEntry, PluginProgEntry}, prework::{Prework, PreworkInFetch, PreworkInLoad, PreworkInSize, PreworkProgFetch, PreworkProgLoad, PreworkProgSize}, process::{Process, ProcessInBg, ProcessInBlock, ProcessInOrphan, ProcessProgBg, ProcessProgBlock, ProcessProgOrphan}};

pub struct Scheduler {
	ops:         TaskOps,
	pub runner:  Runner,
	micro:       async_priority_channel::Sender<TaskIn, u8>,
	handles:     Vec<JoinHandle<()>>,
	pub ongoing: Arc<Mutex<Ongoing>>,
}

impl Scheduler {
	pub fn serve() -> Self {
		let (op_tx, op_rx) = mpsc::unbounded_channel();
		let (micro_tx, micro_rx) = async_priority_channel::unbounded();
		let (macro_tx, macro_rx) = async_priority_channel::unbounded();
		let ongoing = Arc::new(Mutex::new(Ongoing::default()));

		let runner = Runner {
			file:    Arc::new(File::new(&op_tx, &macro_tx)),
			plugin:  Arc::new(Plugin::new(&op_tx, &macro_tx)),
			prework: Arc::new(Prework::new(&op_tx, &macro_tx)),
			process: Arc::new(Process::new(&op_tx)),
			hook:    Arc::new(Hook::new(&op_tx, &ongoing)),
		};

		let mut scheduler = Self {
			ops: TaskOps(op_tx),
			runner,
			micro: micro_tx,
			handles: Vec::with_capacity(
				YAZI.tasks.micro_workers as usize + YAZI.tasks.macro_workers as usize + 1,
			),
			ongoing,
		};

		for _ in 0..YAZI.tasks.micro_workers {
			scheduler.handles.push(scheduler.schedule_micro(micro_rx.clone()));
		}
		for _ in 0..YAZI.tasks.macro_workers {
			scheduler.handles.push(scheduler.schedule_macro(micro_rx.clone(), macro_rx.clone()));
		}
		scheduler.handle_ops(op_rx);
		scheduler
	}

	pub fn cancel(&self, id: Id) -> bool {
		if let Some(hook) = self.ongoing.lock().cancel(id) {
			self.micro.try_send(hook, HIGH).ok();
			return false;
		}

		true
	}

	pub fn shutdown(&self) {
		for handle in &self.handles {
			handle.abort();
		}
	}

	pub fn file_cut(&self, from: UrlBuf, to: UrlBuf, force: bool) {
		let mut ongoing = self.ongoing.lock();
		let task = ongoing.add::<FileProgCut>(format!("Cut {} to {}", from.display(), to.display()));

		if to.try_starts_with(&from).unwrap_or(false) && !to.covariant(&from) {
			return self
				.ops
				.out(task.id, FileOutCut::Fail("Cannot cut directory into itself".to_owned()));
		}

		let follow = !from.scheme().covariant(to.scheme());
		self.queue(
			FileInCut {
				id: task.id,
				from,
				to,
				force,
				cha: None,
				follow,
				retry: 0,
				drop: None,
				done: task.done.clone(),
			},
			LOW,
		);
	}

	pub fn file_copy(&self, from: UrlBuf, to: UrlBuf, force: bool, follow: bool) {
		let mut ongoing = self.ongoing.lock();
		let task = ongoing.add::<FileProgCopy>(format!("Copy {} to {}", from.display(), to.display()));

		if to.try_starts_with(&from).unwrap_or(false) && !to.covariant(&from) {
			return self
				.ops
				.out(task.id, FileOutCopy::Fail("Cannot copy directory into itself".to_owned()));
		}

		let follow = follow || !from.scheme().covariant(to.scheme());
		self.queue(
			FileInCopy {
				id: task.id,
				from,
				to,
				force,
				cha: None,
				follow,
				retry: 0,
				done: task.done.clone(),
			},
			LOW,
		);
	}

	pub fn file_link(&self, from: UrlBuf, to: UrlBuf, relative: bool, force: bool) {
		let mut ongoing = self.ongoing.lock();
		let task = ongoing.add::<FileProgLink>(format!("Link {} to {}", from.display(), to.display()));

		self.queue(
			FileInLink {
				id: task.id,
				from,
				to,
				force,
				cha: None,
				resolve: false,
				relative,
				delete: false,
			},
			LOW,
		);
	}

	pub fn file_hardlink(&self, from: UrlBuf, to: UrlBuf, force: bool, follow: bool) {
		let mut ongoing = self.ongoing.lock();
		let task =
			ongoing.add::<FileProgHardlink>(format!("Hardlink {} to {}", from.display(), to.display()));

		if !from.scheme().covariant(to.scheme()) {
			return self
				.ops
				.out(task.id, FileOutHardlink::Fail("Cannot hardlink cross filesystem".to_owned()));
		}

		if to.try_starts_with(&from).unwrap_or(false) && !to.covariant(&from) {
			return self
				.ops
				.out(task.id, FileOutHardlink::Fail("Cannot hardlink directory into itself".to_owned()));
		}

		self.queue(FileInHardlink { id: task.id, from, to, force, cha: None, follow }, LOW);
	}

	pub fn file_delete(&self, target: UrlBuf) {
		let mut ongoing = self.ongoing.lock();
		let task = ongoing.add::<FileProgDelete>(format!("Delete {}", target.display()));

		task.set_hook(HookInOutDelete { id: task.id, target: target.clone() });
		self.queue(FileInDelete { id: task.id, target, cha: None }, LOW);
	}

	pub fn file_trash(&self, target: UrlBuf) {
		let mut ongoing = self.ongoing.lock();
		let task = ongoing.add::<FileProgTrash>(format!("Trash {}", target.display()));

		task.set_hook(HookInOutTrash { id: task.id, target: target.clone() });
		self.queue(FileInTrash { id: task.id, target }, LOW);
	}

	pub fn file_download(&self, url: UrlBuf) -> CompletionToken {
		let mut ongoing = self.ongoing.lock();
		let task = ongoing.add::<FileProgDownload>(format!("Download {}", url.display()));

		task.set_hook(HookInOutDownload { id: task.id });
		if url.kind().is_remote() {
			self.queue(
				FileInDownload { id: task.id, url, cha: None, retry: 0, done: task.done.clone() },
				LOW,
			);
		} else {
			self.ops.out(task.id, FileOutDownload::Fail("Cannot download non-remote file".to_owned()));
		}

		task.done.clone()
	}

	pub fn file_upload(&self, url: UrlBuf) {
		let mut ongoing = self.ongoing.lock();
		let task = ongoing.add::<FileProgUpload>(format!("Upload {}", url.display()));

		if !url.kind().is_remote() {
			return self
				.ops
				.out(task.id, FileOutUpload::Fail("Cannot upload non-remote file".to_owned()));
		};

		self.queue(
			FileInUpload { id: task.id, url, cha: None, cache: None, done: task.done.clone() },
			LOW,
		);
	}

	pub fn plugin_entry(&self, opt: PluginOpt) {
		let mut ongoing = self.ongoing.lock();
		let task = ongoing.add::<PluginProgEntry>(format!("Run micro plugin `{}`", opt.id));

		self.queue(PluginInEntry { id: task.id, opt }, NORMAL);
	}

	pub fn fetch_paged(
		&self,
		fetcher: &'static Fetcher,
		targets: Vec<yazi_fs::File>,
	) -> CompletionToken {
		let mut ongoing = self.ongoing.lock();
		let task = ongoing.add::<PreworkProgFetch>(format!(
			"Run fetcher `{}` with {} target(s)",
			fetcher.run.name,
			targets.len()
		));

		self.queue(PreworkInFetch { id: task.id, plugin: fetcher, targets }, NORMAL);
		task.done.clone()
	}

	pub async fn fetch_mimetype(&self, targets: Vec<yazi_fs::File>) -> bool {
		let mut wg = vec![];
		for (fetcher, targets) in YAZI.plugin.mime_fetchers(targets) {
			wg.push(self.fetch_paged(fetcher, targets));
		}

		for done in wg {
			if !done.future().await {
				return false; // Canceled
			}
		}
		true
	}

	pub fn preload_paged(&self, preloader: &'static Preloader, target: &yazi_fs::File) {
		let mut ongoing = self.ongoing.lock();
		let task = ongoing.add::<PreworkProgLoad>(format!("Run preloader `{}`", preloader.run.name));

		let target = target.clone();
		self.queue(PreworkInLoad { id: task.id, plugin: preloader, target }, NORMAL);
	}

	pub fn prework_size(&self, targets: Vec<&UrlBuf>) {
		let throttle = Arc::new(Throttle::new(targets.len(), Duration::from_millis(300)));
		let mut ongoing = self.ongoing.lock();

		for target in targets {
			let task =
				ongoing.add::<PreworkProgSize>(format!("Calculate the size of {}", target.display()));
			let target = target.clone();
			let throttle = throttle.clone();

			self.queue(PreworkInSize { id: task.id, target, throttle }, NORMAL);
		}
	}

	pub fn process_open(&self, opt: ProcessOpenOpt) {
		let name = {
			let args = opt.args.iter().map(|a| a.display().to_string()).collect::<Vec<_>>().join(" ");
			if args.is_empty() {
				format!("Run {:?}", opt.cmd)
			} else {
				format!("Run {:?} with `{args}`", opt.cmd)
			}
		};

		let mut ongoing = self.ongoing.lock();
		let task = if opt.block {
			ongoing.add::<ProcessProgBlock>(name)
		} else if opt.orphan {
			ongoing.add::<ProcessProgOrphan>(name)
		} else {
			ongoing.add::<ProcessProgBg>(name)
		};

		if let Some(done) = opt.done {
			task.done = done;
		}

		if opt.block {
			self
				.queue(ProcessInBlock { id: task.id, cwd: opt.cwd, cmd: opt.cmd, args: opt.args }, NORMAL);
		} else if opt.orphan {
			self
				.queue(ProcessInOrphan { id: task.id, cwd: opt.cwd, cmd: opt.cmd, args: opt.args }, NORMAL);
		} else {
			self.queue(
				ProcessInBg {
					id:   task.id,
					cwd:  opt.cwd,
					cmd:  opt.cmd,
					args: opt.args,
					done: task.done.clone(),
				},
				NORMAL,
			);
		};
	}

	fn schedule_micro(&self, rx: async_priority_channel::Receiver<TaskIn, u8>) -> JoinHandle<()> {
		let ops = self.ops.clone();
		let runner = self.runner.clone();
		let ongoing = self.ongoing.clone();

		tokio::spawn(async move {
			loop {
				if let Ok((r#in, _)) = rx.recv().await {
					let id = r#in.id();
					let Some(token) = ongoing.lock().get_token(id) else {
						continue;
					};

					let result = if r#in.is_hook() {
						runner.micro(r#in).await
					} else {
						select! {
							r = runner.micro(r#in) => r,
							false = token.future() => Ok(())
						}
					};

					if let Err(out) = result {
						ops.out(id, out);
					}
				}
			}
		})
	}

	fn schedule_macro(
		&self,
		micro: async_priority_channel::Receiver<TaskIn, u8>,
		r#macro: async_priority_channel::Receiver<TaskIn, u8>,
	) -> JoinHandle<()> {
		let ops = self.ops.clone();
		let runner = self.runner.clone();
		let ongoing = self.ongoing.clone();

		tokio::spawn(async move {
			loop {
				let (r#in, micro) = select! {
					Ok((r#in, _)) = micro.recv() => (r#in, true),
					Ok((r#in, _)) = r#macro.recv() => (r#in, false),
				};

				let id = r#in.id();
				let Some(token) = ongoing.lock().get_token(id) else {
					continue;
				};

				let result = if r#in.is_hook() {
					if micro { runner.micro(r#in).await } else { runner.r#macro(r#in).await }
				} else if micro {
					select! {
						r = runner.micro(r#in) => r,
						false = token.future() => Ok(()),
					}
				} else {
					select! {
						r = runner.r#macro(r#in) => r,
						false = token.future() => Ok(()),
					}
				};

				if let Err(out) = result {
					ops.out(id, out);
				}
			}
		})
	}

	fn handle_ops(&self, mut rx: UnboundedReceiver<TaskOp>) -> JoinHandle<()> {
		let micro = self.micro.clone();
		let ongoing = self.ongoing.clone();

		tokio::spawn(async move {
			while let Some(op) = rx.recv().await {
				let mut ongoing = ongoing.lock();
				let Some(task) = ongoing.get_mut(op.id) else { continue };

				op.out.reduce(task);
				if !task.prog.cooked() && task.done.completed() != Some(false) {
					continue; // Not cooked yet, also not canceled
				} else if task.prog.cleaned() == Some(false) {
					continue; // Failed to clean up
				} else if let Some(hook) = task.hook.take() {
					micro.try_send(hook, LOW).ok();
				} else {
					ongoing.fulfill(op.id);
				}
			}
		})
	}

	#[inline]
	fn queue(&self, r#in: impl Into<TaskIn>, priority: u8) {
		_ = self.micro.try_send(r#in.into(), priority);
	}
}
