use std::{sync::Arc, time::Duration};

use hashbrown::hash_map::RawEntryMut;
use parking_lot::Mutex;
use tokio::{select, sync::{mpsc::{self, UnboundedReceiver}, oneshot}, task::JoinHandle};
use yazi_config::{YAZI, plugin::{Fetcher, Preloader}};
use yazi_parser::{app::PluginOpt, tasks::ProcessOpenOpt};
use yazi_shared::{Id, Throttle, url::{UrlBuf, UrlLike}};

use super::{Ongoing, TaskOp};
use crate::{HIGH, LOW, NORMAL, Runner, TaskIn, TaskOps, file::{File, FileInCopy, FileInCut, FileInDelete, FileInDownload, FileInHardlink, FileInLink, FileInTrash, FileInUpload, FileOutCopy, FileOutCut, FileOutDownload, FileOutHardlink, FileOutUpload, FileProgCopy, FileProgCut, FileProgDelete, FileProgDownload, FileProgHardlink, FileProgLink, FileProgTrash, FileProgUpload}, hook::{Hook, HookInOutBg, HookInOutBlock, HookInOutDelete, HookInOutDownload, HookInOutFetch, HookInOutOrphan, HookInOutTrash}, plugin::{Plugin, PluginInEntry, PluginProgEntry}, prework::{Prework, PreworkInFetch, PreworkInLoad, PreworkInSize, PreworkProgFetch, PreworkProgLoad, PreworkProgSize}, process::{Process, ProcessInBg, ProcessInBlock, ProcessInOrphan, ProcessProgBg, ProcessProgBlock, ProcessProgOrphan}};

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
		let mut ongoing = self.ongoing.lock();

		match ongoing.inner.raw_entry_mut().from_key(&id) {
			RawEntryMut::Occupied(mut oe) => {
				let task = oe.get_mut();
				if let Some(hook) = task.hook.take() {
					task.canceled = true;
					self.micro.try_send(hook, HIGH).ok();
					false
				} else {
					oe.remove();
					true
				}
			}
			RawEntryMut::Vacant(_) => false,
		}
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
			FileInCut { id: task.id, from, to, force, cha: None, follow, retry: 0, drop: None },
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
		self.queue(FileInCopy { id: task.id, from, to, force, cha: None, follow, retry: 0 }, LOW);
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

	pub fn file_download(&self, url: UrlBuf, done: Option<oneshot::Sender<bool>>) {
		let mut ongoing = self.ongoing.lock();
		let task = ongoing.add::<FileProgDownload>(format!("Download {}", url.display()));

		if !url.kind().is_remote() {
			return self
				.ops
				.out(task.id, FileOutDownload::Fail("Cannot download non-remote file".to_owned()));
		};

		if let Some(done) = done {
			task.set_hook(HookInOutDownload { id: task.id, done });
		}

		self.queue(FileInDownload { id: task.id, url, cha: None, retry: 0 }, LOW);
	}

	pub fn file_upload(&self, url: UrlBuf) {
		let mut ongoing = self.ongoing.lock();
		let task = ongoing.add::<FileProgUpload>(format!("Upload {}", url.display()));

		if !url.kind().is_remote() {
			return self
				.ops
				.out(task.id, FileOutUpload::Fail("Cannot upload non-remote file".to_owned()));
		};

		self.queue(FileInUpload { id: task.id, url, cha: None, cache: None }, LOW);
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
		done: Option<oneshot::Sender<bool>>,
	) {
		let mut ongoing = self.ongoing.lock();
		let task = ongoing.add::<PreworkProgFetch>(format!(
			"Run fetcher `{}` with {} target(s)",
			fetcher.run.name,
			targets.len()
		));

		if let Some(done) = done {
			task.set_hook(HookInOutFetch { id: task.id, done });
		}

		self.queue(PreworkInFetch { id: task.id, plugin: fetcher, targets }, NORMAL);
	}

	pub async fn fetch_mimetype(&self, targets: Vec<yazi_fs::File>) -> bool {
		let mut wg = vec![];
		for (fetcher, targets) in YAZI.plugin.mime_fetchers(targets) {
			let (tx, rx) = oneshot::channel();
			self.fetch_paged(fetcher, targets, Some(tx));
			wg.push(rx);
		}

		for rx in wg {
			if rx.await != Ok(true) {
				return false; // Canceled or error
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

		if opt.block {
			task.set_hook(HookInOutBlock { id: task.id, done: opt.done });
			self
				.queue(ProcessInBlock { id: task.id, cwd: opt.cwd, cmd: opt.cmd, args: opt.args }, NORMAL);
		} else if opt.orphan {
			task.set_hook(HookInOutOrphan { id: task.id, done: opt.done });
			self
				.queue(ProcessInOrphan { id: task.id, cwd: opt.cwd, cmd: opt.cmd, args: opt.args }, NORMAL);
		} else {
			let (cancel_tx, cancel_rx) = mpsc::channel(1);
			task.set_hook(HookInOutBg { id: task.id, cancel: cancel_tx, done: opt.done });
			self.queue(
				ProcessInBg {
					id:     task.id,
					cwd:    opt.cwd,
					cmd:    opt.cmd,
					args:   opt.args,
					cancel: cancel_rx,
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
					if !ongoing.lock().exists(id) {
						continue;
					}

					let result = runner.micro(r#in).await;
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
				if !ongoing.lock().exists(id) {
					continue;
				}

				let result = if micro { runner.micro(r#in).await } else { runner.r#macro(r#in).await };
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
				if !task.prog.success() && !task.prog.cleaned() {
					continue;
				} else if let Some(hook) = task.hook.take() {
					micro.try_send(hook, LOW).ok();
				} else {
					ongoing.inner.remove(&op.id);
				}
			}
		})
	}

	#[inline]
	fn queue(&self, r#in: impl Into<TaskIn>, priority: u8) {
		_ = self.micro.try_send(r#in.into(), priority);
	}
}
