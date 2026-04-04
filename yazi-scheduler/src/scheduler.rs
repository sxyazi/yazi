use std::{ops::Deref, sync::Arc, time::Duration};

use hashbrown::HashMap;
use tokio::task::JoinHandle;
use yazi_config::{YAZI, plugin::{Fetcher, Preloader}};
use yazi_runner::entry::EntryJob;
use yazi_shared::{CompletionToken, Id, SStr, Throttle, data::{Data, DataKey}, url::{UrlBuf, UrlLike}};

use crate::{Behavior, HIGH, LOW, NORMAL, Task, TaskProg, Worker, fetch::{FetchIn, FetchProg}, file::{FileInCopy, FileInCut, FileInDelete, FileInDownload, FileInHardlink, FileInLink, FileInTrash, FileInUpload, FileOutCopy, FileOutCut, FileOutDownload, FileOutHardlink, FileOutUpload, FileProgCopy, FileProgCut, FileProgDelete, FileProgDownload, FileProgHardlink, FileProgLink, FileProgTrash, FileProgUpload}, hook::{HookIn, HookInDelete, HookInDownload, HookInTrash, HookInUpload}, plugin::{PluginInEntry, PluginProgEntry}, preload::{PreloadIn, PreloadProg}, process::{ProcessInBg, ProcessInBlock, ProcessInOrphan, ProcessOpt, ProcessProgBg, ProcessProgBlock, ProcessProgOrphan}, size::{SizeIn, SizeProg}};

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

	fn add<T, R>(&self, name: String, map: impl FnOnce(&mut Task) -> R) -> R
	where
		T: Into<TaskProg> + Default,
	{
		let prog = T::default().into();

		let mut ongoing = self.ongoing.lock();
		let task = ongoing.add(name, prog);

		self.behavior.update(task.id);
		map(task)
	}

	fn add_hooked<T, R>(
		&self,
		name: String,
		hook: impl Into<HookIn>,
		map: impl FnOnce(&mut Task) -> R,
	) -> R
	where
		T: Into<TaskProg> + Default,
	{
		self.add::<T, R>(name, |t| map(t.with_hook(hook)))
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
		let name = format!("Cut {} to {}", from.display(), to.display());
		let id = self.add::<FileProgCut, _>(name, |t| t.id);

		if to.try_starts_with(&from).unwrap_or(false) && !to.covariant(&from) {
			return self.ops.out(id, FileOutCut::Fail("Cannot cut directory into itself".to_owned()));
		}

		let follow = !from.scheme().covariant(to.scheme());
		self
			.file
			.submit(FileInCut { id, from, to, force, cha: None, follow, retry: 0, drop: None }, LOW);
	}

	pub fn file_copy(&self, from: UrlBuf, to: UrlBuf, force: bool, follow: bool) {
		let name = format!("Copy {} to {}", from.display(), to.display());
		let id = self.add::<FileProgCopy, _>(name, |t| t.id);

		if to.try_starts_with(&from).unwrap_or(false) && !to.covariant(&from) {
			return self.ops.out(id, FileOutCopy::Fail("Cannot copy directory into itself".to_owned()));
		}

		let follow = follow || !from.scheme().covariant(to.scheme());
		self.file.submit(FileInCopy { id, from, to, force, cha: None, follow, retry: 0 }, LOW);
	}

	pub fn file_link(&self, from: UrlBuf, to: UrlBuf, relative: bool, force: bool) {
		let name = format!("Link {} to {}", from.display(), to.display());
		let id = self.add::<FileProgLink, _>(name, |t| t.id);

		self.file.submit(
			FileInLink { id, from, to, force, cha: None, resolve: false, relative, delete: false },
			LOW,
		);
	}

	pub fn file_hardlink(&self, from: UrlBuf, to: UrlBuf, force: bool, follow: bool) {
		let name = format!("Hardlink {} to {}", from.display(), to.display());
		let id = self.add::<FileProgHardlink, _>(name, |t| t.id);

		if !from.scheme().covariant(to.scheme()) {
			return self
				.ops
				.out(id, FileOutHardlink::Fail("Cannot hardlink cross filesystem".to_owned()));
		}

		if to.try_starts_with(&from).unwrap_or(false) && !to.covariant(&from) {
			return self
				.ops
				.out(id, FileOutHardlink::Fail("Cannot hardlink directory into itself".to_owned()));
		}

		self.file.submit(FileInHardlink { id, from, to, force, cha: None, follow }, LOW);
	}

	pub fn file_delete(&self, target: UrlBuf) {
		let name = format!("Delete {}", target.display());

		let hook = HookInDelete::new(&target);
		let id = self.add_hooked::<FileProgDelete, _>(name, hook, |t| t.id);

		self.file.submit(FileInDelete { id, target, cha: None }, LOW);
	}

	pub fn file_trash(&self, target: UrlBuf) {
		let name = format!("Trash {}", target.display());

		let hook = HookInTrash::new(&target);
		let id = self.add_hooked::<FileProgTrash, _>(name, hook, |t| t.id);

		self.file.submit(FileInTrash { id, target }, LOW);
	}

	pub fn file_download(&self, target: UrlBuf) -> CompletionToken {
		let name = format!("Download {}", target.display());

		let hook = HookInDownload::new(&target);
		let (id, done) = self.add_hooked::<FileProgDownload, _>(name, hook, |t| (t.id, t.done.clone()));

		if !target.kind().is_remote() {
			self.ops.out(id, FileOutDownload::Fail("Cannot download non-remote file".to_owned()));
			return done;
		}

		self.file.submit(FileInDownload { id, target, cha: None, retry: 0 }, LOW);
		done
	}

	pub fn file_upload(&self, target: UrlBuf) {
		let name = format!("Upload {}", target.display());

		let hook = HookInUpload::new(&target);
		let id = self.add_hooked::<FileProgUpload, _>(name, hook, |t| t.id);

		if !target.kind().is_remote() {
			return self.ops.out(id, FileOutUpload::Fail("Cannot upload non-remote file".to_owned()));
		}

		self.file.submit(FileInUpload { id, target, cha: None, cache: None }, LOW);
	}

	pub fn plugin_entry(&self, plugin: SStr, args: HashMap<DataKey, Data>) {
		let name = format!("Run micro plugin `{plugin}`");
		let id = self.add::<PluginProgEntry, _>(name, |t| t.id);

		self.plugin.submit(PluginInEntry(EntryJob { id, args, plugin }), NORMAL);
	}

	pub fn fetch_paged(
		&self,
		fetcher: &'static Fetcher,
		targets: Vec<yazi_fs::File>,
	) -> CompletionToken {
		let name = format!("Run fetcher `{}` with {} target(s)", fetcher.run.name, targets.len());
		let (id, done) = self.add::<FetchProg, _>(name, |t| (t.id, t.done.clone()));

		self.fetch.submit(FetchIn { id, plugin: fetcher, targets });
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
		let name = format!("Run preloader `{}`", preloader.run.name);
		let (id, done) = self.add::<PreloadProg, _>(name, |t| (t.id, t.done.clone()));

		self.preload.submit(PreloadIn { id, plugin: preloader, target: target.clone(), done });
	}

	pub fn prework_size(&self, targets: Vec<&UrlBuf>) {
		let throttle = Arc::new(Throttle::new(targets.len(), Duration::from_millis(300)));

		for target in targets {
			let name = format!("Calculate the size of {}", target.display());
			let id = self.add::<SizeProg, _>(name, |t| t.id);

			self.size.submit(SizeIn { id, target: target.clone(), throttle: throttle.clone() }, NORMAL);
		}
	}

	pub fn process_open(&self, opt: ProcessOpt) -> CompletionToken {
		let name = {
			let args = opt.args.iter().map(|a| a.display().to_string()).collect::<Vec<_>>().join(" ");
			if args.is_empty() {
				format!("Run {:?}", opt.cmd)
			} else {
				format!("Run {:?} with `{args}`", opt.cmd)
			}
		};

		let (id, done) = if opt.block {
			self.add::<ProcessProgBlock, _>(name, |t| (t.id, t.done.clone()))
		} else if opt.orphan {
			self.add::<ProcessProgOrphan, _>(name, |t| (t.id, t.done.clone()))
		} else {
			self.add::<ProcessProgBg, _>(name, |t| (t.id, t.done.clone()))
		};

		if opt.block {
			self
				.process
				.submit(ProcessInBlock { id, cwd: opt.cwd, cmd: opt.cmd, args: opt.args }, NORMAL);
		} else if opt.orphan {
			self
				.process
				.submit(ProcessInOrphan { id, cwd: opt.cwd, cmd: opt.cmd, args: opt.args }, NORMAL);
		} else {
			self.process.submit(
				ProcessInBg { id, cwd: opt.cwd, cmd: opt.cmd, args: opt.args, done: done.clone() },
				NORMAL,
			);
		};

		done
	}
}
