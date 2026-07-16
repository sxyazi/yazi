use yazi_config::{YAZI, plugin::MAX_FETCHERS};
use yazi_fs::{Entries, FsHash64, SortBy, file::File};

use super::Tasks;
use crate::mgr::Mimetype;

impl Tasks {
	pub fn fetch_paged(&self, paged: &[File], mimetype: &Mimetype) {
		let mut loaded = self.scheduler.fetch.loaded.lock();
		let mut tasks: [Vec<_>; MAX_FETCHERS as usize] = Default::default();
		for f in paged {
			let hash = f.hash_u64();
			for g in YAZI.plugin.fetchers.matches(f, mimetype.get(&f.url).unwrap_or_default()) {
				match loaded.get_mut(&hash) {
					Some(n) if *n & (1 << g.idx) != 0 => continue,
					Some(n) => *n |= 1 << g.idx,
					None => _ = loaded.put(hash, 1 << g.idx),
				}
				tasks[g.idx as usize].push(f.clone());
			}
		}

		drop(loaded);
		let fetchers = YAZI.plugin.fetchers.load();
		for (i, tasks) in tasks.into_iter().enumerate() {
			if !tasks.is_empty() {
				self.scheduler.fetch_paged(fetchers[i].clone(), tasks);
			}
		}
	}

	pub fn preload_paged(&self, paged: &[File], mimetype: &Mimetype) {
		let mut loaded = self.scheduler.preload.loaded.lock();
		for f in paged {
			let hash = f.hash_u64();
			for p in YAZI.plugin.preloaders.matches(f, mimetype.get(&f.url).unwrap_or_default()) {
				match loaded.get_mut(&hash) {
					Some(n) if *n & (1 << p.idx) != 0 => continue,
					Some(n) => *n |= 1 << p.idx,
					None => _ = loaded.put(hash, 1 << p.idx),
				}
				self.scheduler.preload_paged(p, f);
			}
		}
	}

	pub fn prework_sorted(&self, targets: &Entries) {
		if targets.sorter().by != SortBy::Size {
			return;
		}

		let targets: Vec<_> = {
			let loading = self.scheduler.size.sizing.read();
			targets
				.iter()
				.filter(|f| {
					let key = f.entry_key();
					f.is_dir()
						&& !key.is_empty()
						&& !targets.sizes.contains_key(&key)
						&& !loading.contains(&f.url)
				})
				.map(|f| &f.url)
				.collect()
		};
		if targets.is_empty() {
			return;
		}

		let mut loading = self.scheduler.size.sizing.write();
		for &target in &targets {
			loading.insert(target.clone());
		}

		self.scheduler.prework_size(targets);
	}
}
