use yazi_config::{YAZI, plugin::MAX_PREWORKERS};
use yazi_fs::{File, Files, SortBy};

use super::Tasks;
use crate::mgr::Mimetype;

impl Tasks {
	pub fn fetch_paged(&self, paged: &[File], mimetype: &Mimetype) {
		let mut loaded = self.scheduler.prework.loaded.lock();
		let mut tasks: [Vec<_>; MAX_PREWORKERS as usize] = Default::default();
		for f in paged {
			let hash = f.hash_u64();
			for g in YAZI.plugin.fetchers(&f.url, mimetype.by_file(f).unwrap_or_default()) {
				match loaded.get_mut(&hash) {
					Some(n) if *n & (1 << g.idx) != 0 => continue,
					Some(n) => *n |= 1 << g.idx,
					None => _ = loaded.put(hash, 1 << g.idx),
				}
				tasks[g.idx as usize].push(f.clone());
			}
		}

		drop(loaded);
		for (i, tasks) in tasks.into_iter().enumerate() {
			if !tasks.is_empty() {
				self.scheduler.fetch_paged(&YAZI.plugin.fetchers[i], tasks);
			}
		}
	}

	pub fn preload_paged(&self, paged: &[File], mimetype: &Mimetype) {
		let mut loaded = self.scheduler.prework.loaded.lock();
		for f in paged {
			let hash = f.hash_u64();
			for p in YAZI.plugin.preloaders(&f.url, mimetype.by_file(f).unwrap_or_default()) {
				match loaded.get_mut(&hash) {
					Some(n) if *n & (1 << p.idx) != 0 => continue,
					Some(n) => *n |= 1 << p.idx,
					None => _ = loaded.put(hash, 1 << p.idx),
				}
				self.scheduler.preload_paged(p, f);
			}
		}
	}

	pub fn prework_sorted(&self, targets: &Files) {
		if targets.sorter().by != SortBy::Size {
			return;
		}

		let targets: Vec<_> = {
			let loading = self.scheduler.prework.sizing.read();
			targets
				.iter()
				.filter(|f| f.is_dir() && !targets.sizes.contains_key(f.urn()) && !loading.contains(&f.url))
				.map(|f| &f.url)
				.collect()
		};
		if targets.is_empty() {
			return;
		}

		let mut loading = self.scheduler.prework.sizing.write();
		for &target in &targets {
			loading.insert(target.clone());
		}

		self.scheduler.prework_size(targets);
	}
}
