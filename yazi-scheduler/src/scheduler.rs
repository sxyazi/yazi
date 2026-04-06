use std::{ops::Deref, sync::Arc, time::Duration};

use tokio::task::JoinHandle;
use yazi_config::{YAZI, plugin::{Fetcher, Preloader}};
use yazi_fs::FsHash64;
use yazi_shared::{CompletionToken, Id, Throttle, url::{UrlBuf, UrlLike}};

use crate::{Behavior, HIGH, LOW, NORMAL, Task, TaskIn, TaskProg, Worker, fetch::FetchIn, file::{FileInCopy, FileInCut, FileInDelete, FileInDownload, FileInHardlink, FileInLink, FileInTrash, FileInUpload, FileOutCopy, FileOutCut, FileOutDownload, FileOutHardlink, FileOutUpload}, hook::{HookIn, HookInDelete, HookInDownload, HookInPreload, HookInTrash, HookInUpload}, plugin::PluginInEntry, preload::PreloadIn, process::{ProcessIn, ProcessInBg, ProcessInBlock, ProcessInOrphan, ProcessOpt}, size::SizeIn};

pub struct Scheduler {
	pub worker:   Worker,
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

	pub fn file_cut(&self, from: UrlBuf, to: UrlBuf, force: bool) {
		let follow = !from.scheme().covariant(to.scheme());
		let mut r#in =
			FileInCut { id: Id::ZERO, from, to, force, cha: None, follow, retry: 0, drop: None };

		self.add(&mut r#in, |_| ());
		if r#in.to.try_starts_with(&r#in.from).unwrap_or(false) && !r#in.to.covariant(&r#in.from) {
			self.ops.out(r#in.id, FileOutCut::Fail("Cannot cut directory into itself".to_owned()));
		} else {
			self.file.submit(r#in, LOW);
		}
	}

	pub fn file_copy(&self, from: UrlBuf, to: UrlBuf, force: bool, follow: bool) {
		let follow = follow || !from.scheme().covariant(to.scheme());
		let mut r#in = FileInCopy { id: Id::ZERO, from, to, force, cha: None, follow, retry: 0 };

		self.add(&mut r#in, |_| ());
		if r#in.to.try_starts_with(&r#in.from).unwrap_or(false) && !r#in.to.covariant(&r#in.from) {
			self.ops.out(r#in.id, FileOutCopy::Fail("Cannot copy directory into itself".to_owned()));
		} else {
			self.file.submit(r#in, LOW);
		}
	}

	pub fn file_link(&self, from: UrlBuf, to: UrlBuf, relative: bool, force: bool) {
		let mut r#in = FileInLink {
			id: Id::ZERO,
			from,
			to,
			force,
			cha: None,
			resolve: false,
			relative,
			delete: false,
		};

		self.add(&mut r#in, |_| ());
		self.file.submit(r#in, LOW);
	}

	pub fn file_hardlink(&self, from: UrlBuf, to: UrlBuf, force: bool, follow: bool) {
		let mut r#in = FileInHardlink { id: Id::ZERO, from, to, force, cha: None, follow };
		self.add(&mut r#in, |_| ());

		if !r#in.from.scheme().covariant(r#in.to.scheme()) {
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

	pub fn file_download(&self, target: UrlBuf) -> CompletionToken {
		let mut r#in = FileInDownload { id: Id::ZERO, target, cha: None, retry: 0 };
		let hook = HookInDownload::new(&r#in.target);
		let done = self.add_hooked(&mut r#in, hook, |t| t.done.clone());

		if r#in.target.kind().is_remote() {
			self.file.submit(r#in, LOW);
		} else {
			self.ops.out(r#in.id, FileOutDownload::Fail("Cannot download non-remote file".to_owned()));
		}
		done
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

	pub fn plugin_entry(&self, mut r#in: PluginInEntry) -> Id {
		if r#in.track {
			self.behavior.reset();
		}

		let id = self.add(&mut r#in, |t| t.id);
		self.plugin.submit(r#in, NORMAL);

		id
	}

	pub fn fetch_paged(
		&self,
		fetcher: &'static Fetcher,
		targets: Vec<yazi_fs::File>,
	) -> CompletionToken {
		let mut r#in = FetchIn { id: Id::ZERO, plugin: fetcher, targets };

		let done = self.add(&mut r#in, |t| t.done.clone());
		self.fetch.submit(r#in);

		done
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
		let mut r#in = PreloadIn { id: Id::ZERO, plugin: preloader, target: target.clone() };
		let hook = HookInPreload::new(preloader.idx, target.hash_u64());

		self.add_hooked(&mut r#in, hook, |_| ());
		if let Some(prev) = self.preload.loading.lock().put(target.url.hash_u64(), r#in.id) {
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

	pub fn process_open(&self, opt: ProcessOpt) -> CompletionToken {
		let mut r#in: ProcessIn = if opt.block {
			ProcessInBlock { id: Id::ZERO, cwd: opt.cwd, cmd: opt.cmd, args: opt.args }.into()
		} else if opt.orphan {
			ProcessInOrphan { id: Id::ZERO, cwd: opt.cwd, cmd: opt.cmd, args: opt.args }.into()
		} else {
			ProcessInBg {
				id:   Id::ZERO,
				cwd:  opt.cwd,
				cmd:  opt.cmd,
				args: opt.args,
				done: CompletionToken::default(),
			}
			.into()
		};

		let done = match &mut r#in {
			ProcessIn::Block(r#in) => self.add(r#in, |t| t.done.clone()),
			ProcessIn::Orphan(r#in) => self.add(r#in, |t| t.done.clone()),
			ProcessIn::Bg(r#in) => {
				r#in.done = self.add(r#in, |t| t.done.clone());
				r#in.done.clone()
			}
		};

		self.process.submit(r#in, NORMAL);
		done
	}
}
