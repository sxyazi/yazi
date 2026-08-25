use std::{ops::Deref, sync::Arc, time::Duration};

use tokio::{select, task::JoinHandle};
use yazi_config::{YAZI, plugin::{FetcherArc, PreloaderArc}};
use yazi_fs::{FsHash64, file::{File, FileSig}};
use yazi_shared::{Throttle, id::Id, pool::Symbol, url::{UrlBuf, UrlLike}};

use crate::{Behavior, HIGH, LOW, NORMAL, Task, TaskHandle, TaskIn, TaskProg, Worker, custom::{CustomIn, CustomOut, CustomPool}, fetch::FetchInFetch, file::{FileInCopy, FileInDelete, FileInDownload, FileInHardlink, FileInLink, FileInMove, FileInTrash, FileInUpload, FileOutCopy, FileOutDownload, FileOutHardlink, FileOutMove, FileOutUpload}, hook::{HookIn, HookInDelete, HookInDownload, HookInPreload, HookInTrash, HookInUpload}, plugin::PluginInEntry, preload::PreloadInPreload, process::{ProcessIn, ProcessInBg, ProcessInBlock, ProcessInOrphan, ShellOpt}, size::SizeIn};

pub struct Scheduler {
	worker:       Worker,
	pub behavior: Behavior,
	handles:      Vec<JoinHandle<()>>,
}

impl Deref for Scheduler {
	type Target = Worker;

	fn deref(&self) -> &Self::Target { &self.worker }
}

impl Scheduler {
	pub fn serve() -> Self {
		let (worker, handles) = Worker::make();
		Self { worker, behavior: Behavior::new(), handles }
	}

	fn add<T, R>(&self, r#in: &mut T, map: impl FnOnce(&mut Task) -> R) -> R
	where
		T: TaskIn,
		T::Prog: Into<TaskProg> + Default,
	{
		let mut ongoing = self.ongoing.lock();
		let task = ongoing.add(r#in);

		self.behavior.update(task.id);
		map(task)
	}

	fn add_hooked<T, R>(
		&self,
		r#in: &mut T,
		hook: impl Into<HookIn>,
		map: impl FnOnce(&mut Task) -> R,
	) -> R
	where
		T: TaskIn,
		T::Prog: Into<TaskProg> + Default,
	{
		self.add(r#in, |t| map(t.with_hook(hook)))
	}

	pub fn cancel(&self, id: Id) -> bool {
		if let Some(hook) = self.ongoing.lock().cancel(id) {
			self.hook.submit(hook, HIGH);
			return false;
		}

		true
	}

	pub fn shutdown(&self) {
		for handle in &self.handles {
			handle.abort();
		}
	}

	pub fn file_move(&self, mut r#in: FileInMove) -> TaskHandle {
		let handle = self.add(&mut r#in, |t| t.handle.clone());

		if r#in.to.try_starts_with(&r#in.from).unwrap_or(false) && !r#in.to.covariant(&r#in.from) {
			self.ops.out(r#in.id, FileOutMove::Fail("Cannot move directory into itself".to_owned()));
		} else {
			self.file.submit(r#in, LOW);
		}

		handle
	}

	pub fn file_copy(&self, mut r#in: FileInCopy) -> TaskHandle {
		let handle = self.add(&mut r#in, |t| t.handle.clone());

		if r#in.to.try_starts_with(&r#in.from).unwrap_or(false) && !r#in.to.covariant(&r#in.from) {
			self.ops.out(r#in.id, FileOutCopy::Fail("Cannot copy directory into itself".to_owned()));
		} else {
			self.file.submit(r#in, LOW);
		}

		handle
	}

	pub fn file_link(&self, mut r#in: FileInLink) {
		self.add(&mut r#in, |_| ());
		self.file.submit(r#in, LOW);
	}

	pub fn file_hardlink(&self, from: UrlBuf, to: UrlBuf, force: bool, follow: bool) {
		let mut r#in = FileInHardlink { id: Id::ZERO, from, to, force, cha: None, follow };
		self.add(&mut r#in, |_| ());

		if !r#in.from.auth().same_service(r#in.to.auth()) {
			return self
				.ops
				.out(r#in.id, FileOutHardlink::Fail("Cannot hardlink cross filesystem".to_owned()));
		}

		if r#in.to.try_starts_with(&r#in.from).unwrap_or(false) && !r#in.to.covariant(&r#in.from) {
			return self
				.ops
				.out(r#in.id, FileOutHardlink::Fail("Cannot hardlink directory into itself".to_owned()));
		}

		self.file.submit(r#in, LOW);
	}

	pub fn file_delete(&self, target: UrlBuf) {
		let mut r#in = FileInDelete { id: Id::ZERO, target, cha: None };
		let hook = HookInDelete::new(&r#in.target);

		self.add_hooked(&mut r#in, hook, |_| ());
		self.file.submit(r#in, LOW);
	}

	pub fn file_trash(&self, target: UrlBuf) {
		let mut r#in = FileInTrash { id: Id::ZERO, target };
		let hook = HookInTrash::new(&r#in.target);

		self.add_hooked(&mut r#in, hook, |_| ());
		self.file.submit(r#in, LOW);
	}

	pub fn file_download(&self, target: UrlBuf) -> TaskHandle {
		let mut r#in = FileInDownload { id: Id::ZERO, target, cha: None, retry: 0 };
		let hook = HookInDownload::new(&r#in.target);
		let handle = self.add_hooked(&mut r#in, hook, |t| t.handle.clone());

		if r#in.target.kind().is_remote() {
			self.file.submit(r#in, LOW);
		} else {
			self.ops.out(r#in.id, FileOutDownload::Fail("Cannot download non-remote file".to_owned()));
		}
		handle
	}

	pub fn file_upload(&self, target: UrlBuf) {
		let mut r#in = FileInUpload { id: Id::ZERO, target, cha: None, cache: None };
		let hook = HookInUpload::new(&r#in.target);
		self.add_hooked(&mut r#in, hook, |_| ());

		if r#in.target.kind().is_remote() {
			self.file.submit(r#in, LOW);
		} else {
			self.ops.out(r#in.id, FileOutUpload::Fail("Cannot upload non-remote file".to_owned()));
		}
	}

	pub fn plugin_entry(&self, mut r#in: PluginInEntry) -> TaskHandle {
		if r#in.track {
			self.behavior.reset();
		}

		let handle = self.add(&mut r#in, |t| t.handle.clone());
		self.plugin.submit(r#in, NORMAL);

		handle
	}

	pub fn fetch_paged(&self, fetcher: FetcherArc, targets: Vec<File>) -> TaskHandle {
		let mut r#in = FetchInFetch { id: Id::ZERO, fetcher, targets };

		let handle = self.add(&mut r#in, |t| t.handle.clone());
		self.fetch.submit(r#in);

		handle
	}

	pub async fn fetch_mimetype(&self, targets: Vec<File>) -> bool {
		let mut wg = vec![];
		for (fetcher, targets) in YAZI.plugin.fetchers.mime(targets) {
			wg.push(self.fetch_paged(fetcher, targets));
		}

		for done in wg {
			if !done.future().await {
				return false; // Failed or canceled
			}
		}
		true
	}

	pub fn preload_paged(&self, preloader: PreloaderArc, file: &File, mime: Symbol<str>) {
		let hook = HookInPreload::new(&preloader, FileSig(file).hash_u64());
		let mut r#in = PreloadInPreload { id: Id::ZERO, preloader, file: file.clone(), mime };

		self.add_hooked(&mut r#in, hook, |_| ());
		if let Some(prev) = self.preload.loading.lock().put(file.url.hash_u64(), r#in.id) {
			self.cancel(prev);
		}

		self.preload.submit(r#in);
	}

	pub fn prework_size(&self, targets: Vec<&UrlBuf>) {
		let throttle = Arc::new(Throttle::new(targets.len(), Duration::from_millis(300)));

		for target in targets {
			let mut r#in =
				SizeIn { id: Id::ZERO, target: target.clone(), throttle: throttle.clone() };

			self.add(&mut r#in, |_| ());
			self.size.submit(r#in, NORMAL);
		}
	}

	pub fn process_open(&self, opt: ShellOpt) -> TaskHandle {
		let mut r#in: ProcessIn = if opt.block {
			ProcessInBlock { id: Id::ZERO, cwd: opt.cwd, cmd: opt.cmd }.into()
		} else if opt.orphan {
			ProcessInOrphan { id: Id::ZERO, cwd: opt.cwd, cmd: opt.cmd }.into()
		} else {
			ProcessInBg { id: Id::ZERO, cwd: opt.cwd, cmd: opt.cmd }.into()
		};

		let handle = match &mut r#in {
			ProcessIn::Block(r#in) => self.add(r#in, |t| t.handle.clone()),
			ProcessIn::Orphan(r#in) => self.add(r#in, |t| t.handle.clone()),
			ProcessIn::Bg(r#in) => self.add(r#in, |t| t.handle.clone()),
			ProcessIn::Custom(_) => unreachable!(),
		};

		self.process.submit(r#in, NORMAL);
		handle
	}

	pub fn custom(&self, mut r#in: CustomIn) -> TaskHandle {
		if r#in.track {
			self.behavior.reset();
		}

		let progress = r#in.progress;
		let handle = self.add(&mut r#in, |t| {
			let TaskProg::Custom(prog) = &mut t.prog else { unreachable!() };
			prog.progress = progress;
			t.handle.clone()
		});

		let ongoing = self.ongoing.clone();
		let scope = r#in.scope.clone();
		let handle_ = handle.clone();
		tokio::spawn(async move {
			select! {
				_ = scope.cancelled() => {
					ongoing.lock().cancel(handle_.id);
				}
				status = handle_.finished() => if status.is_canceled() {
					scope.cancel();
				}
			}
		});

		match r#in.pool {
			CustomPool::File => self.file.submit(r#in, LOW),
			CustomPool::Plugin => self.plugin.submit(r#in, LOW),
			CustomPool::Fetch => self.fetch.submit(r#in),
			CustomPool::Preload => self.preload.submit(r#in),
			CustomPool::Process => self.process.submit(r#in, LOW),
			CustomPool::None => handle.start(),
		}

		handle
	}

	pub fn custom_output(&self, id: Id, out: CustomOut) { self.ops.out(id, out); }
}
