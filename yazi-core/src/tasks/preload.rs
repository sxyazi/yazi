use std::collections::HashMap;

use yazi_config::{manager::SortBy, plugin::MAX_PREWORKERS, PLUGIN};
use yazi_shared::{fs::{File, Url}, MIME_DIR};

use super::Tasks;
use crate::folder::Files;

impl Tasks {
	pub fn fetch_paged(&self, paged: &[File], mimetype: &HashMap<Url, String>) {
		let mut loaded = self.scheduler.prework.loaded.lock();
		let mut tasks: [Vec<_>; MAX_PREWORKERS as usize] = Default::default();
		for f in paged {
			let mime = if f.is_dir() { Some(MIME_DIR) } else { mimetype.get(&f.url).map(|s| &**s) };
			let factors = |s: &str| match s {
				"mime" => mime.is_some(),
				_ => false,
			};

			for p in PLUGIN.fetchers(&f.url, mime, factors) {
				match loaded.get_mut(&f.url) {
					Some(n) if *n & (1 << p.idx) != 0 => continue,
					Some(n) => *n |= 1 << p.idx,
					None => _ = loaded.insert(f.url.clone(), 1 << p.idx),
				}
				tasks[p.idx as usize].push(f.clone());
			}
		}

		drop(loaded);
		for (i, tasks) in tasks.into_iter().enumerate() {
			if !tasks.is_empty() {
				self.scheduler.fetch_paged(&PLUGIN.fetchers[i], tasks);
			}
		}
	}

	pub fn preload_paged(&self, paged: &[File], mimetype: &HashMap<Url, String>) {
		let mut loaded = self.scheduler.prework.loaded.lock();
		for f in paged {
			let mime = if f.is_dir() { Some(MIME_DIR) } else { mimetype.get(&f.url).map(|s| &**s) };
			for p in PLUGIN.preloaders(&f.url, mime) {
				match loaded.get_mut(&f.url) {
					Some(n) if *n & (1 << p.idx) != 0 => continue,
					Some(n) => *n |= 1 << p.idx,
					None => _ = loaded.insert(f.url.clone(), 1 << p.idx),
				}
				self.scheduler.preload_paged(p, f);
			}
		}
	}

	pub fn prework_affected(&self, affected: &[File], mimetype: &HashMap<Url, String>) {
		{
			let mut loaded = self.scheduler.prework.loaded.lock();
			for f in affected {
				loaded.remove(&f.url);
			}
		}

		self.fetch_paged(affected, mimetype);
		self.preload_paged(affected, mimetype);
	}

	pub fn prework_sorted(&self, targets: &Files) {
		if targets.sorter().by != SortBy::Size {
			return;
		}

		let targets: Vec<_> = {
			let loading = self.scheduler.prework.size_loading.read();
			targets
				.iter()
				.filter(|f| f.is_dir() && !targets.sizes.contains_key(&f.url) && !loading.contains(&f.url))
				.map(|f| &f.url)
				.collect()
		};
		if targets.is_empty() {
			return;
		}

		let mut loading = self.scheduler.prework.size_loading.write();
		for &target in &targets {
			loading.insert(target.clone());
		}

		self.scheduler.prework_size(targets);
	}
}
