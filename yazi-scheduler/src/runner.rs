use std::sync::Arc;

use parking_lot::Mutex;
use tokio::{select, sync::mpsc, task::JoinHandle};
use yazi_config::YAZI;

use crate::{LOW, Ongoing, TaskOp, TaskOps, TaskOut, fetch::{Fetch, FetchIn}, file::{File, FileIn}, hook::{Hook, HookIn}, plugin::{Plugin, PluginIn}, preload::{Preload, PreloadIn}, process::{Process, ProcessIn}, size::{Size, SizeIn}};

#[derive(Clone)]
pub struct Runner {
	pub(super) file:    Arc<File>,
	pub(super) plugin:  Arc<Plugin>,
	pub fetch:          Arc<Fetch>,
	pub preload:        Arc<Preload>,
	pub size:           Arc<Size>,
	pub(super) process: Arc<Process>,
	pub(super) hook:    Arc<Hook>,

	pub ops:     TaskOps,
	pub ongoing: Arc<Mutex<Ongoing>>,
}

impl Runner {
	pub(super) fn make() -> (Self, Vec<JoinHandle<()>>) {
		let (file_tx, file_rx) = async_priority_channel::unbounded();
		let (plugin_tx, plugin_rx) = async_priority_channel::unbounded();
		let (fetch_tx, fetch_rx) = async_priority_channel::unbounded();
		let (preload_tx, preload_rx) = async_priority_channel::unbounded();
		let (size_tx, size_rx) = async_priority_channel::unbounded();
		let (process_tx, process_rx) = async_priority_channel::unbounded();
		let (hook_tx, hook_rx) = async_priority_channel::unbounded();
		let (op_tx, op_rx) = mpsc::unbounded_channel();
		let ongoing = Arc::new(Mutex::new(Ongoing::default()));

		let me = Self {
			file: Arc::new(File::new(&op_tx, file_tx)),
			plugin: Arc::new(Plugin::new(&op_tx, plugin_tx)),
			fetch: Arc::new(Fetch::new(&op_tx, fetch_tx)),
			preload: Arc::new(Preload::new(&op_tx, preload_tx)),
			size: Arc::new(Size::new(&op_tx, size_tx)),
			process: Arc::new(Process::new(&op_tx, process_tx)),
			hook: Arc::new(Hook::new(&op_tx, &ongoing, hook_tx)),

			ops: TaskOps(op_tx),
			ongoing,
		};

		let handles = []
			.into_iter()
			.chain((0..YAZI.tasks.file_workers).map(|_| me.file(file_rx.clone())))
			.chain((0..YAZI.tasks.plugin_workers).map(|_| me.plugin(plugin_rx.clone())))
			.chain((0..YAZI.tasks.fetch_workers).map(|_| me.fetch(fetch_rx.clone())))
			.chain((0..YAZI.tasks.preload_workers).map(|_| me.preload(preload_rx.clone())))
			.chain((0..3).map(|_| me.size(size_rx.clone())))
			.chain((0..YAZI.tasks.process_workers).map(|_| me.process(process_rx.clone())))
			.chain((0..3).map(|_| me.hook(hook_rx.clone())))
			.chain([me.op(op_rx)])
			.collect();

		(me, handles)
	}

	fn file(&self, rx: async_priority_channel::Receiver<FileIn, u8>) -> JoinHandle<()> {
		let me = self.clone();
		tokio::spawn(async move {
			loop {
				if let Ok((r#in, _)) = rx.recv().await {
					let id = r#in.id();
					let Some(token) = me.ongoing.lock().get_token(id) else {
						continue;
					};

					let result = select! {
						r = me.file_do(r#in) => r,
						false = token.future() => Ok(())
					};

					if let Err(out) = result {
						me.ops.out(id, out);
					}
				}
			}
		})
	}

	async fn file_do(&self, r#in: FileIn) -> Result<(), TaskOut> {
		match r#in {
			FileIn::Copy(r#in) => self.file.copy(r#in).await.map_err(Into::into),
			FileIn::CopyDo(r#in) => self.file.copy_do(r#in).await.map_err(Into::into),
			FileIn::Cut(r#in) => self.file.cut(r#in).await.map_err(Into::into),
			FileIn::CutDo(r#in) => self.file.cut_do(r#in).await.map_err(Into::into),
			FileIn::Link(r#in) => self.file.link(r#in).await.map_err(Into::into),
			FileIn::LinkDo(r#in) => self.file.link_do(r#in).await.map_err(Into::into),
			FileIn::Hardlink(r#in) => self.file.hardlink(r#in).await.map_err(Into::into),
			FileIn::HardlinkDo(r#in) => self.file.hardlink_do(r#in).await.map_err(Into::into),
			FileIn::Delete(r#in) => self.file.delete(r#in).await.map_err(Into::into),
			FileIn::DeleteDo(r#in) => self.file.delete_do(r#in).await.map_err(Into::into),
			FileIn::Trash(r#in) => self.file.trash(r#in).await.map_err(Into::into),
			FileIn::TrashDo(r#in) => self.file.trash_do(r#in).await.map_err(Into::into),
			FileIn::Download(r#in) => self.file.download(r#in).await.map_err(Into::into),
			FileIn::DownloadDo(r#in) => self.file.download_do(r#in).await.map_err(Into::into),
			FileIn::Upload(r#in) => self.file.upload(r#in).await.map_err(Into::into),
			FileIn::UploadDo(r#in) => self.file.upload_do(r#in).await.map_err(Into::into),
		}
	}

	fn plugin(&self, rx: async_priority_channel::Receiver<PluginIn, u8>) -> JoinHandle<()> {
		let me = self.clone();
		tokio::spawn(async move {
			loop {
				if let Ok((r#in, _)) = rx.recv().await {
					let id = r#in.id();
					let Some(token) = me.ongoing.lock().get_token(id) else {
						continue;
					};

					let result = select! {
						r = me.plugin_do(r#in) => r,
						false = token.future() => Ok(())
					};

					if let Err(out) = result {
						me.ops.out(id, out);
					}
				}
			}
		})
	}

	async fn plugin_do(&self, r#in: PluginIn) -> Result<(), TaskOut> {
		match r#in {
			PluginIn::Entry(r#in) => self.plugin.entry(r#in).await.map_err(Into::into),
		}
	}

	fn fetch(&self, rx: async_priority_channel::Receiver<FetchIn, u8>) -> JoinHandle<()> {
		let me = self.clone();
		tokio::spawn(async move {
			loop {
				if let Ok((r#in, _)) = rx.recv().await {
					let id = r#in.id();
					let Some(token) = me.ongoing.lock().get_token(id) else {
						continue;
					};

					let result = select! {
						r = me.fetch_do(r#in) => r,
						false = token.future() => Ok(())
					};

					if let Err(out) = result {
						me.ops.out(id, out);
					}
				}
			}
		})
	}

	async fn fetch_do(&self, r#in: FetchIn) -> Result<(), TaskOut> {
		self.fetch.fetch(r#in).await.map_err(Into::into)
	}

	fn preload(&self, rx: async_priority_channel::Receiver<PreloadIn, u8>) -> JoinHandle<()> {
		let me = self.clone();
		tokio::spawn(async move {
			loop {
				if let Ok((r#in, _)) = rx.recv().await {
					let id = r#in.id();
					let Some(token) = me.ongoing.lock().get_token(id) else {
						continue;
					};

					let result = select! {
						r = me.preload_do(r#in) => r,
						false = token.future() => Ok(())
					};

					if let Err(out) = result {
						me.ops.out(id, out);
					}
				}
			}
		})
	}

	async fn preload_do(&self, r#in: PreloadIn) -> Result<(), TaskOut> {
		self.preload.preload(r#in).await.map_err(Into::into)
	}

	fn size(&self, rx: async_priority_channel::Receiver<SizeIn, u8>) -> JoinHandle<()> {
		let me = self.clone();
		tokio::spawn(async move {
			loop {
				if let Ok((r#in, _)) = rx.recv().await {
					let id = r#in.id();
					let Some(token) = me.ongoing.lock().get_token(id) else {
						continue;
					};

					let result = select! {
						r = me.size_do(r#in) => r,
						false = token.future() => Ok(())
					};

					if let Err(out) = result {
						me.ops.out(id, out);
					}
				}
			}
		})
	}

	async fn size_do(&self, r#in: SizeIn) -> Result<(), TaskOut> {
		self.size.size(r#in).await.map_err(Into::into)
	}

	fn process(&self, rx: async_priority_channel::Receiver<ProcessIn, u8>) -> JoinHandle<()> {
		let me = self.clone();
		tokio::spawn(async move {
			loop {
				if let Ok((r#in, _)) = rx.recv().await {
					let id = r#in.id();
					let Some(token) = me.ongoing.lock().get_token(id) else {
						continue;
					};

					let result = select! {
						r = me.process_do(r#in) => r,
						false = token.future() => Ok(())
					};

					if let Err(out) = result {
						me.ops.out(id, out);
					}
				}
			}
		})
	}

	async fn process_do(&self, r#in: ProcessIn) -> Result<(), TaskOut> {
		match r#in {
			ProcessIn::Block(r#in) => self.process.block(r#in).await.map_err(Into::into),
			ProcessIn::Orphan(r#in) => self.process.orphan(r#in).await.map_err(Into::into),
			ProcessIn::Bg(r#in) => self.process.bg(r#in).await.map_err(Into::into),
		}
	}

	fn hook(&self, rx: async_priority_channel::Receiver<HookIn, u8>) -> JoinHandle<()> {
		let me = self.clone();
		tokio::spawn(async move {
			loop {
				if let Ok((r#in, _)) = rx.recv().await {
					let id = r#in.id();
					if !me.ongoing.lock().exists(id) {
						continue;
					}
					me.hook_do(r#in).await;
				}
			}
		})
	}

	async fn hook_do(&self, r#in: HookIn) {
		match r#in {
			HookIn::Copy(r#in) => self.hook.copy(r#in).await,
			HookIn::Cut(r#in) => self.hook.cut(r#in).await,
			HookIn::Delete(r#in) => self.hook.delete(r#in).await,
			HookIn::Trash(r#in) => self.hook.trash(r#in).await,
			HookIn::Download(r#in) => self.hook.download(r#in).await,
			HookIn::Upload(r#in) => self.hook.upload(r#in).await,
		}
	}

	fn op(&self, mut rx: mpsc::UnboundedReceiver<TaskOp>) -> JoinHandle<()> {
		let me = self.clone();
		tokio::spawn(async move {
			while let Some(op) = rx.recv().await {
				let mut ongoing = me.ongoing.lock();
				let Some(task) = ongoing.get_mut(op.id) else { continue };

				op.out.reduce(task);
				if !task.prog.cooked() && task.done.completed() != Some(false) {
					continue; // Not cooked yet, also not canceled
				} else if task.prog.cleaned() == Some(false) {
					continue; // Failed to clean up
				} else if let Some(hook) = task.hook.take() {
					me.hook.submit(hook, LOW);
				} else {
					ongoing.fulfill(op.id);
				}
			}
		})
	}
}
