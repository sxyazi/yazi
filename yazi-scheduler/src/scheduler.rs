use std::{ops::Deref, sync::Arc, time::Duration};

use tokio::task::JoinHandle;
use yazi_config::{YAZI, plugin::{Fetcher, Preloader}};
use yazi_parser::{app::PluginOpt, tasks::ProcessOpenOpt};
use yazi_shared::{CompletionToken, Id, Throttle, url::{UrlBuf, UrlLike}};

use crate::{HIGH, LOW, NORMAL, Runner, fetch::{FetchIn, FetchProg}, file::{FileInCopy, FileInCut, FileInDelete, FileInDownload, FileInHardlink, FileInLink, FileInTrash, FileInUpload, FileOutCopy, FileOutCut, FileOutDownload, FileOutHardlink, FileOutUpload, FileProgCopy, FileProgCut, FileProgDelete, FileProgDownload, FileProgHardlink, FileProgLink, FileProgTrash, FileProgUpload}, hook::{HookInDelete, HookInDownload, HookInTrash, HookInUpload}, plugin::{PluginInEntry, PluginProgEntry}, preload::{PreloadIn, PreloadProg}, process::{ProcessInBg, ProcessInBlock, ProcessInOrphan, ProcessProgBg, ProcessProgBlock, ProcessProgOrphan}, size::{SizeIn, SizeProg}};

pub struct Scheduler {
	pub runner: Runner,
	handles:    Vec<JoinHandle<()>>,
}

impl Deref for Scheduler {
	type Target = Runner;

	fn deref(&self) -> &Self::Target { &self.runner }
}

impl Scheduler {
	pub fn serve() -> Self {
		let (runner, handles) = Runner::make();
		Self { runner, handles }
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
		let mut ongoing = self.ongoing.lock();
		let task = ongoing.add::<FileProgCut>(format!("Cut {} to {}", from.display(), to.display()));

		if to.try_starts_with(&from).unwrap_or(false) && !to.covariant(&from) {
			return self
				.ops
				.out(task.id, FileOutCut::Fail("Cannot cut directory into itself".to_owned()));
		}

		let follow = !from.scheme().covariant(to.scheme());
		self.file.submit(
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
		self.file.submit(
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

		self.file.submit(
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

		self.file.submit(FileInHardlink { id: task.id, from, to, force, cha: None, follow }, LOW);
	}

	pub fn file_delete(&self, target: UrlBuf) {
		let mut ongoing = self.ongoing.lock();
		let task = ongoing.add::<FileProgDelete>(format!("Delete {}", target.display()));

		task.set_hook(HookInDelete { id: task.id, target: target.clone() });
		self.file.submit(FileInDelete { id: task.id, target, cha: None }, LOW);
	}

	pub fn file_trash(&self, target: UrlBuf) {
		let mut ongoing = self.ongoing.lock();
		let task = ongoing.add::<FileProgTrash>(format!("Trash {}", target.display()));

		task.set_hook(HookInTrash { id: task.id, target: target.clone() });
		self.file.submit(FileInTrash { id: task.id, target }, LOW);
	}

	pub fn file_download(&self, target: UrlBuf) -> CompletionToken {
		let mut ongoing = self.ongoing.lock();
		let task = ongoing.add::<FileProgDownload>(format!("Download {}", target.display()));

		if target.kind().is_remote() {
			task.set_hook(HookInDownload { id: task.id, target: target.clone() });
			self.file.submit(
				FileInDownload { id: task.id, target, cha: None, retry: 0, done: task.done.clone() },
				LOW,
			);
		} else {
			self.ops.out(task.id, FileOutDownload::Fail("Cannot download non-remote file".to_owned()));
		}

		task.done.clone()
	}

	pub fn file_upload(&self, target: UrlBuf) {
		let mut ongoing = self.ongoing.lock();
		let task = ongoing.add::<FileProgUpload>(format!("Upload {}", target.display()));

		if !target.kind().is_remote() {
			return self
				.ops
				.out(task.id, FileOutUpload::Fail("Cannot upload non-remote file".to_owned()));
		};

		task.set_hook(HookInUpload { id: task.id, target: target.clone() });
		self.file.submit(
			FileInUpload { id: task.id, target, cha: None, cache: None, done: task.done.clone() },
			LOW,
		);
	}

	pub fn plugin_entry(&self, opt: PluginOpt) {
		let mut ongoing = self.ongoing.lock();
		let task = ongoing.add::<PluginProgEntry>(format!("Run micro plugin `{}`", opt.id));

		self.plugin.submit(PluginInEntry { id: task.id, opt }, NORMAL);
	}

	pub fn fetch_paged(
		&self,
		fetcher: &'static Fetcher,
		targets: Vec<yazi_fs::File>,
	) -> CompletionToken {
		let mut ongoing = self.ongoing.lock();
		let task = ongoing.add::<FetchProg>(format!(
			"Run fetcher `{}` with {} target(s)",
			fetcher.run.name,
			targets.len()
		));

		self.fetch.submit(FetchIn { id: task.id, plugin: fetcher, targets });
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
		let task = ongoing.add::<PreloadProg>(format!("Run preloader `{}`", preloader.run.name));

		let target = target.clone();
		self.preload.submit(PreloadIn { id: task.id, plugin: preloader, target });
	}

	pub fn prework_size(&self, targets: Vec<&UrlBuf>) {
		let throttle = Arc::new(Throttle::new(targets.len(), Duration::from_millis(300)));
		let mut ongoing = self.ongoing.lock();

		for target in targets {
			let task = ongoing.add::<SizeProg>(format!("Calculate the size of {}", target.display()));
			let target = target.clone();
			let throttle = throttle.clone();

			self.size.submit(SizeIn { id: task.id, target, throttle }, NORMAL);
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
				.process
				.submit(ProcessInBlock { id: task.id, cwd: opt.cwd, cmd: opt.cmd, args: opt.args }, NORMAL);
		} else if opt.orphan {
			self.process.submit(
				ProcessInOrphan { id: task.id, cwd: opt.cwd, cmd: opt.cmd, args: opt.args },
				NORMAL,
			);
		} else {
			self.process.submit(
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
}
