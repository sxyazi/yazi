use std::{collections::{BTreeMap, HashMap, HashSet}, ffi::OsStr, mem, path::Path, sync::Arc, time::Duration};

use tokio::time::sleep;
use tracing::debug;
use yazi_config::{manager::SortBy, open::Opener, plugin::{PluginRule, MAX_PRELOADERS}, popup::InputCfg, OPEN, PLUGIN};
use yazi_plugin::ValueSendable;
use yazi_scheduler::{Scheduler, TaskSummary};
use yazi_shared::{emit, event::Exec, fs::{File, Url}, term::Term, Layer, MIME_DIR};

use super::{TasksProgress, TASKS_PADDING, TASKS_PERCENT};
use crate::{folder::Files, input::Input};

pub struct Tasks {
	pub(super) scheduler: Arc<Scheduler>,

	pub visible:  bool,
	pub cursor:   usize,
	pub progress: TasksProgress,
}

impl Tasks {
	pub fn start() -> Self {
		let tasks = Self {
			scheduler: Arc::new(Scheduler::start()),
			visible:   false,
			cursor:    0,
			progress:  Default::default(),
		};

		let running = tasks.scheduler.running.clone();
		tokio::spawn(async move {
			let mut last = TasksProgress::default();
			loop {
				sleep(Duration::from_millis(500)).await;

				let new = TasksProgress::from(&*running.lock());
				if last != new {
					last = new;
					emit!(Call(Exec::call("update_progress", vec![]).with_data(new), Layer::App));
				}
			}
		});

		tasks
	}

	#[inline]
	pub fn limit() -> usize {
		(Term::size().rows * TASKS_PERCENT / 100).saturating_sub(TASKS_PADDING) as usize
	}

	pub fn paginate(&self) -> Vec<TaskSummary> {
		let running = self.scheduler.running.lock();
		running.values().take(Self::limit()).map(Into::into).collect()
	}

	pub fn file_open(&self, targets: &[(impl AsRef<Path>, impl AsRef<str>)]) -> bool {
		let mut openers = BTreeMap::new();
		for (path, mime) in targets {
			if let Some(opener) = OPEN.openers(path, mime).and_then(|o| o.first().copied()) {
				openers.entry(opener).or_insert_with(Vec::new).push(path.as_ref().as_os_str());
			}
		}
		for (opener, args) in openers {
			self.file_open_with(opener, &args);
		}
		false
	}

	pub fn file_open_with(&self, opener: &Opener, args: &[impl AsRef<OsStr>]) -> bool {
		if opener.spread {
			self.scheduler.process_open(opener, args);
			return false;
		}
		for target in args {
			self.scheduler.process_open(opener, &[target]);
		}
		false
	}

	pub fn file_cut(&self, src: &HashSet<Url>, dest: &Url, force: bool) {
		for u in src {
			let to = dest.join(u.file_name().unwrap());
			if force && u == &to {
				debug!("file_cut: same file, skipping {:?}", to);
			} else {
				self.scheduler.file_cut(u.clone(), to, force);
			}
		}
	}

	pub fn file_copy(&self, src: &HashSet<Url>, dest: &Url, force: bool, follow: bool) {
		for u in src {
			let to = dest.join(u.file_name().unwrap());
			if force && u == &to {
				debug!("file_copy: same file, skipping {:?}", to);
			} else {
				self.scheduler.file_copy(u.clone(), to, force, follow);
			}
		}
	}

	pub fn file_link(&self, src: &HashSet<Url>, dest: &Url, relative: bool, force: bool) {
		for u in src {
			let to = dest.join(u.file_name().unwrap());
			if force && *u == to {
				debug!("file_link: same file, skipping {:?}", to);
			} else {
				self.scheduler.file_link(u.clone(), to, relative, force);
			}
		}
	}

	pub fn file_remove(&self, targets: Vec<Url>, force: bool, permanently: bool) {
		if force {
			for u in targets {
				if permanently {
					self.scheduler.file_delete(u);
				} else {
					self.scheduler.file_trash(u);
				}
			}
			return;
		}

		let scheduler = self.scheduler.clone();
		tokio::spawn(async move {
			let mut result = Input::_show(if permanently {
				InputCfg::delete(targets.len())
			} else {
				InputCfg::trash(targets.len())
			});

			if let Some(Ok(choice)) = result.recv().await {
				if choice != "y" && choice != "Y" {
					return;
				}
				for u in targets {
					if permanently {
						scheduler.file_delete(u);
					} else {
						scheduler.file_trash(u);
					}
				}
			}
		});
	}

	#[inline]
	pub fn plugin_micro(&self, name: String, args: Vec<ValueSendable>) {
		self.scheduler.plugin_micro(name, args);
	}

	#[inline]
	pub fn plugin_macro(&self, name: String, args: Vec<ValueSendable>) {
		self.scheduler.plugin_macro(name, args);
	}

	pub fn preload_paged(&self, paged: &[File], mimetype: &HashMap<Url, String>) {
		let mut single_tasks = Vec::with_capacity(paged.len());
		let mut multi_tasks: [Vec<_>; MAX_PRELOADERS as usize] = Default::default();

		let loaded = self.scheduler.preload.rule_loaded.read();
		for f in paged {
			let mime = if f.is_dir() { Some(MIME_DIR) } else { mimetype.get(&f.url).map(|s| &**s) };
			let factors = |s: &str| match s {
				"mime" => mime.is_some(),
				_ => false,
			};

			for rule in PLUGIN.preloaders(&f.url, mime, factors) {
				if loaded.get(&f.url).is_some_and(|x| x & (1 << rule.id) != 0) {
					continue;
				}
				if rule.multi {
					multi_tasks[rule.id as usize].push(f);
				} else {
					single_tasks.push((rule, f));
				}
			}
		}

		drop(loaded);
		let mut loaded = self.scheduler.preload.rule_loaded.write();

		let mut go = |rule: &PluginRule, targets: Vec<&File>| {
			for &f in &targets {
				if let Some(n) = loaded.get_mut(&f.url) {
					*n |= 1 << rule.id;
				} else {
					loaded.insert(f.url.clone(), 1 << rule.id);
				}
			}
			self.scheduler.preload_paged(rule, targets);
		};

		#[allow(clippy::needless_range_loop)]
		for i in 0..PLUGIN.preloaders.len() {
			if !multi_tasks[i].is_empty() {
				go(&PLUGIN.preloaders[i], mem::take(&mut multi_tasks[i]));
			}
		}
		for (rule, target) in single_tasks {
			go(rule, vec![target]);
		}
	}

	pub fn preload_affected(&self, affected: &[File], mimetype: &HashMap<Url, String>) {
		{
			let mut loaded = self.scheduler.preload.rule_loaded.write();
			for f in affected {
				loaded.remove(&f.url);
			}
		}

		self.preload_paged(affected, mimetype);
	}

	pub fn preload_sorted(&self, targets: &Files) {
		if targets.sorter().by != SortBy::Size {
			return;
		}

		let targets: Vec<_> = {
			let loading = self.scheduler.preload.size_loading.read();
			targets
				.iter()
				.filter(|f| f.is_dir() && !targets.sizes.contains_key(&f.url) && !loading.contains(&f.url))
				.map(|f| &f.url)
				.collect()
		};
		if targets.is_empty() {
			return;
		}

		let mut loading = self.scheduler.preload.size_loading.write();
		for &target in &targets {
			loading.insert(target.clone());
		}

		self.scheduler.preload_size(targets);
	}
}

impl Tasks {
	#[inline]
	pub fn len(&self) -> usize { self.scheduler.running.lock().len() }
}
