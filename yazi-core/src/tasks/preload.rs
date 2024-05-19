use std::{collections::HashMap, mem};

use yazi_config::{manager::SortBy, plugin::{Preloader, MAX_PRELOADERS}, PLUGIN};
use yazi_shared::{fs::{File, Url}, MIME_DIR};

use super::Tasks;
use crate::folder::Files;

impl Tasks {
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

		let mut go = |preloader: &Preloader, targets: Vec<&File>| {
			for &f in &targets {
				if let Some(n) = loaded.get_mut(&f.url) {
					*n |= 1 << preloader.id;
				} else {
					loaded.insert(f.url.clone(), 1 << preloader.id);
				}
			}
			self.scheduler.preload_paged(preloader, targets);
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
