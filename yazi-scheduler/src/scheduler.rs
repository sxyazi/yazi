use std::{future::Future, sync::Arc, time::Duration};

use anyhow::Result;
use futures::{FutureExt, future::BoxFuture};
use parking_lot::Mutex;
use tokio::{select, sync::{mpsc::{self, UnboundedReceiver}, oneshot}, task::JoinHandle};
use yazi_config::{YAZI, plugin::{Fetcher, Preloader}};
use yazi_dds::Pump;
use yazi_parser::{app::PluginOpt, tasks::ProcessOpenOpt};
use yazi_proxy::TasksProxy;
use yazi_shared::{Id, Throttle, url::{UrlBuf, UrlLike}};
use yazi_vfs::{must_be_dir, provider, unique_name};

use super::{Ongoing, TaskOp};
use crate::{HIGH, LOW, NORMAL, TaskIn, TaskOps, TaskOut, file::{File, FileInDelete, FileInDownload, FileInHardlink, FileInLink, FileInPaste, FileInTrash, FileInUpload, FileOutDelete, FileOutDownload, FileOutHardlink, FileOutPaste, FileOutUpload, FileProgDelete, FileProgDownload, FileProgHardlink, FileProgLink, FileProgPaste, FileProgTrash, FileProgUpload}, plugin::{Plugin, PluginInEntry, PluginProgEntry}, prework::{Prework, PreworkInFetch, PreworkInLoad, PreworkInSize, PreworkProgFetch, PreworkProgLoad, PreworkProgSize}, process::{Process, ProcessInBg, ProcessInBlock, ProcessInOrphan, ProcessOutBg, ProcessOutBlock, ProcessOutOrphan, ProcessProgBg, ProcessProgBlock, ProcessProgOrphan}};

pub struct Scheduler {
	file:        Arc<File>,
	plugin:      Arc<Plugin>,
	pub prework: Arc<Prework>,
	process:     Arc<Process>,

	ops:         TaskOps,
	micro:       async_priority_channel::Sender<BoxFuture<'static, ()>, u8>,
	handles:     Vec<JoinHandle<()>>,
	pub ongoing: Arc<Mutex<Ongoing>>,
}

impl Scheduler {
	pub fn serve() -> Self {
		let (op_tx, op_rx) = mpsc::unbounded_channel();
		let (micro_tx, micro_rx) = async_priority_channel::unbounded();
		let (macro_tx, macro_rx) = async_priority_channel::unbounded();

		let mut scheduler = Self {
			file:    Arc::new(File::new(&op_tx, &macro_tx)),
			plugin:  Arc::new(Plugin::new(&op_tx, &macro_tx)),
			prework: Arc::new(Prework::new(&op_tx, &macro_tx)),
			process: Arc::new(Process::new(&op_tx)),

			ops:     TaskOps(op_tx),
			micro:   micro_tx,
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
		scheduler.handle_ops(op_rx);
		scheduler
	}

	pub fn cancel(&self, id: Id) -> bool {
		let mut ongoing = self.ongoing.lock();

		if let Some(hook) = ongoing.hooks.pop(id)
			&& let Some(fut) = hook.call(true)
		{
			self.micro.try_send(fut, HIGH).ok();
			return false;
		}

		ongoing.all.remove(&id).is_some()
	}

	pub fn shutdown(&self) {
		for handle in &self.handles {
			handle.abort();
		}
	}

	pub fn file_cut(&self, from: UrlBuf, mut to: UrlBuf, force: bool) {
		let mut ongoing = self.ongoing.lock();
		let id = ongoing.add::<FileProgPaste>(format!("Cut {} to {}", from.display(), to.display()));

		let Ok(prefixed) = to.try_starts_with(&from) else {
			return self
				.ops
				.out(id, FileOutPaste::Fail("Path being cut has a different encoding".to_owned()));
		};

		if prefixed && !to.covariant(&from) {
			return self.ops.out(id, FileOutPaste::Fail("Cannot cut directory into itself".to_owned()));
		}

		ongoing.hooks.add_async(id, {
			let ops = self.ops.clone();
			let (from, to) = (from.clone(), to.clone());

			move |canceled| async move {
				if !canceled {
					provider::remove_dir_clean(&from).await.ok();
					Pump::push_move(from, to);
				}
				ops.out(id, FileOutPaste::Clean);
			}
		});

		let file = self.file.clone();
		self.send_micro(id, LOW, async move {
			if !force {
				to = unique_name(to, must_be_dir(&from)).await?;
			}
			file.paste(FileInPaste { id, from, to, cha: None, cut: true, follow: false, retry: 0 }).await
		});
	}

	pub fn file_copy(&self, from: UrlBuf, mut to: UrlBuf, force: bool, follow: bool) {
		let id = self.ongoing.lock().add::<FileProgPaste>(format!(
			"Copy {} to {}",
			from.display(),
			to.display()
		));

		let Ok(prefixed) = to.try_starts_with(&from) else {
			return self
				.ops
				.out(id, FileOutPaste::Fail("Path being copied has a different encoding".to_owned()));
		};

		if prefixed && !to.covariant(&from) {
			return self.ops.out(id, FileOutPaste::Fail("Cannot copy directory into itself".to_owned()));
		}

		let file = self.file.clone();
		self.send_micro(id, LOW, async move {
			if !force {
				to = unique_name(to, must_be_dir(&from)).await?;
			}
			file.paste(FileInPaste { id, from, to, cha: None, cut: false, follow, retry: 0 }).await
		});
	}

	pub fn file_link(&self, from: UrlBuf, mut to: UrlBuf, relative: bool, force: bool) {
		let id = self.ongoing.lock().add::<FileProgLink>(format!(
			"Link {} to {}",
			from.display(),
			to.display()
		));

		let file = self.file.clone();
		self.send_micro(id, LOW, async move {
			if !force {
				to = unique_name(to, must_be_dir(&from)).await?;
			}
			file.link(FileInLink { id, from, to, cha: None, resolve: false, relative, delete: false })
		});
	}

	pub fn file_hardlink(&self, from: UrlBuf, mut to: UrlBuf, force: bool, follow: bool) {
		let id = self.ongoing.lock().add::<FileProgHardlink>(format!(
			"Hardlink {} to {}",
			from.display(),
			to.display()
		));

		let Ok(prefixed) = to.try_starts_with(&from) else {
			return self.ops.out(
				id,
				FileOutHardlink::Fail("Path being hardlinked has a different encoding".to_owned()),
			);
		};

		if prefixed && !to.covariant(&from) {
			return self
				.ops
				.out(id, FileOutHardlink::Fail("Cannot hardlink directory into itself".to_owned()));
		}

		let file = self.file.clone();
		self.send_micro(id, LOW, async move {
			if !force {
				to = unique_name(to, must_be_dir(&from)).await?;
			}
			file.hardlink(FileInHardlink { id, from, to, cha: None, follow }).await
		});
	}

	pub fn file_delete(&self, target: UrlBuf) {
		let mut ongoing = self.ongoing.lock();
		let id = ongoing.add::<FileProgDelete>(format!("Delete {}", target.display()));

		ongoing.hooks.add_async(id, {
			let ops = self.ops.clone();
			let target = target.clone();

			move |canceled| async move {
				if !canceled {
					provider::remove_dir_all(&target).await.ok();
					TasksProxy::update_succeed(&target);
					Pump::push_delete(target);
				}
				ops.out(id, FileOutDelete::Clean);
			}
		});

		let file = self.file.clone();
		self.send_micro(
			id,
			LOW,
			async move { file.delete(FileInDelete { id, target, length: 0 }).await },
		);
	}

	pub fn file_trash(&self, target: UrlBuf) {
		let mut ongoing = self.ongoing.lock();
		let id = ongoing.add::<FileProgTrash>(format!("Trash {}", target.display()));

		ongoing.hooks.add_sync(id, {
			let target = target.clone();
			move |canceled| {
				if !canceled {
					TasksProxy::update_succeed(&target);
					Pump::push_trash(target);
				}
			}
		});

		let file = self.file.clone();
		self.send_micro(id, LOW, async move { file.trash(FileInTrash { id, target }) })
	}

	pub fn file_download(&self, url: UrlBuf, done: Option<oneshot::Sender<bool>>) {
		let mut ongoing = self.ongoing.lock();
		let id = ongoing.add::<FileProgDownload>(format!("Download {}", url.display()));

		if let Some(tx) = done {
			ongoing.hooks.add_sync(id, move |canceled| _ = tx.send(canceled));
		}

		if !url.kind().is_remote() {
			return self.ops.out(id, FileOutDownload::Fail("Cannot download non-remote file".to_owned()));
		};

		let file = self.file.clone();
		self.send_micro(id, LOW, async move {
			file.download(FileInDownload { id, url, cha: None, retry: 0 }).await
		});
	}

	pub fn file_upload(&self, url: UrlBuf) {
		let mut ongoing = self.ongoing.lock();
		let id = ongoing.add::<FileProgUpload>(format!("Upload {}", url.display()));

		if !url.kind().is_remote() {
			return self.ops.out(id, FileOutUpload::Fail("Cannot upload non-remote file".to_owned()));
		};

		let file = self.file.clone();
		self.send_micro(id, LOW, async move { file.upload(FileInUpload { id, url }).await });
	}

	pub fn plugin_micro(&self, opt: PluginOpt) {
		let id = self.ongoing.lock().add::<PluginProgEntry>(format!("Run micro plugin `{}`", opt.id));

		let plugin = self.plugin.clone();
		self.send_micro(id, NORMAL, async move { plugin.micro(PluginInEntry { id, opt }).await });
	}

	pub fn plugin_macro(&self, opt: PluginOpt) {
		let id = self.ongoing.lock().add::<PluginProgEntry>(format!("Run macro plugin `{}`", opt.id));

		self.plugin.r#macro(PluginInEntry { id, opt }).ok();
	}

	pub fn fetch_paged(
		&self,
		fetcher: &'static Fetcher,
		targets: Vec<yazi_fs::File>,
		done: Option<oneshot::Sender<bool>>,
	) {
		let mut ongoing = self.ongoing.lock();
		let id = ongoing.add::<PreworkProgFetch>(format!(
			"Run fetcher `{}` with {} target(s)",
			fetcher.run.name,
			targets.len()
		));

		if let Some(tx) = done {
			ongoing.hooks.add_sync(id, move |canceled| _ = tx.send(canceled));
		}

		let prework = self.prework.clone();
		self.send_micro(id, NORMAL, async move {
			prework.fetch(PreworkInFetch { id, plugin: fetcher, targets }).await
		});
	}

	pub async fn fetch_mimetype(&self, targets: Vec<yazi_fs::File>) -> bool {
		let mut wg = vec![];
		for (fetcher, targets) in YAZI.plugin.mime_fetchers(targets) {
			let (tx, rx) = oneshot::channel();
			self.fetch_paged(fetcher, targets, Some(tx));
			wg.push(rx);
		}

		for rx in wg {
			if rx.await != Ok(false) {
				return false; // Canceled or error
			}
		}
		true
	}

	pub fn preload_paged(&self, preloader: &'static Preloader, target: &yazi_fs::File) {
		let id =
			self.ongoing.lock().add::<PreworkProgLoad>(format!("Run preloader `{}`", preloader.run.name));

		let target = target.clone();
		let prework = self.prework.clone();
		self.send_micro(id, NORMAL, async move {
			prework.load(PreworkInLoad { id, plugin: preloader, target }).await
		});
	}

	pub fn prework_size(&self, targets: Vec<&UrlBuf>) {
		let throttle = Arc::new(Throttle::new(targets.len(), Duration::from_millis(300)));
		let mut ongoing = self.ongoing.lock();

		for target in targets {
			let id =
				ongoing.add::<PreworkProgSize>(format!("Calculate the size of {}", target.display()));
			let target = target.clone();
			let throttle = throttle.clone();

			let prework = self.prework.clone();
			self.send_micro(id, NORMAL, async move {
				prework.size(PreworkInSize { id, target, throttle }).await
			});
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
		let (id, clean): (_, TaskOut) = if opt.block {
			(ongoing.add::<ProcessProgBlock>(name), ProcessOutBlock::Clean.into())
		} else if opt.orphan {
			(ongoing.add::<ProcessProgOrphan>(name), ProcessOutOrphan::Clean.into())
		} else {
			(ongoing.add::<ProcessProgBg>(name), ProcessOutBg::Clean.into())
		};

		let ops = self.ops.clone();
		let (cancel_tx, cancel_rx) = mpsc::channel(1);
		ongoing.hooks.add_async(id, move |canceled| async move {
			if canceled {
				cancel_tx.send(()).await.ok();
				cancel_tx.closed().await;
			}
			if let Some(tx) = opt.done {
				tx.send(()).ok();
			}
			ops.out(id, clean);
		});

		let process = self.process.clone();
		self.send_micro::<_, TaskOut>(id, NORMAL, async move {
			if opt.block {
				process.block(ProcessInBlock { id, cwd: opt.cwd, cmd: opt.cmd, args: opt.args }).await?;
			} else if opt.orphan {
				process.orphan(ProcessInOrphan { id, cwd: opt.cwd, cmd: opt.cmd, args: opt.args }).await?;
			} else {
				process
					.bg(ProcessInBg { id, cwd: opt.cwd, cmd: opt.cmd, args: opt.args, cancel: cancel_rx })
					.await?;
			}
			Ok(())
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
		r#macro: async_priority_channel::Receiver<TaskIn, u8>,
	) -> JoinHandle<()> {
		let file = self.file.clone();
		let plugin = self.plugin.clone();
		let prework = self.prework.clone();

		let ops = self.ops.clone();
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

						let result: Result<_, TaskOut> = match r#in {
							// File
							TaskIn::FilePaste(r#in) => file.paste_do(r#in).await.map_err(Into::into),
							TaskIn::FileLink(r#in) => file.link_do(r#in).await.map_err(Into::into),
							TaskIn::FileHardlink(r#in) => file.hardlink_do(r#in).await.map_err(Into::into),
							TaskIn::FileDelete(r#in) => file.delete_do(r#in).await.map_err(Into::into),
							TaskIn::FileTrash(r#in) => file.trash_do(r#in).await.map_err(Into::into),
							TaskIn::FileDownload(r#in) => file.download_do(r#in).await.map_err(Into::into),
							TaskIn::FileUploadDo(r#in) => file.upload_do(r#in).await.map_err(Into::into),
							// Plugin
							TaskIn::PluginEntry(r#in) => plugin.macro_do(r#in).await.map_err(Into::into),
							// Prework
							TaskIn::PreworkFetch(r#in) => prework.fetch_do(r#in).await.map_err(Into::into),
							TaskIn::PreworkLoad(r#in) => prework.load_do(r#in).await.map_err(Into::into),
							TaskIn::PreworkSize(r#in) => prework.size_do(r#in).await.map_err(Into::into),
						};

						if let Err(out) = result {
							ops.out(id, out);
						}
					}
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
				} else if let Some(hook) = ongoing.hooks.pop(op.id)
					&& let Some(fut) = hook.call(false)
				{
					micro.try_send(fut, LOW).ok();
				} else {
					ongoing.all.remove(&op.id);
				}
			}
		})
	}

	fn send_micro<F, E>(&self, id: Id, priority: u8, f: F)
	where
		F: Future<Output = Result<(), E>> + Send + 'static,
		E: Into<TaskOut>,
	{
		let ops = self.ops.clone();
		_ = self.micro.try_send(
			async move {
				if let Err(out) = f.await {
					ops.out(id, out);
				}
			}
			.boxed(),
			priority,
		);
	}
}
