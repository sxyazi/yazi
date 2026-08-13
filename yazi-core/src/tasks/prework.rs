use yazi_config::{YAZI, plugin::{FetcherMatcher, MAX_FETCHERS}};
use yazi_fs::{Entries, FsHash64, SortBy, file::{File, FileSig}};
use yazi_scheduler::Loaded;
use yazi_shared::pool::InternStr;

use super::Tasks;
use crate::mgr::Mimetype;

impl Tasks {
	pub fn fetch_paged(&self, paged: &[File], mimetype: &Mimetype) {
		let fetchers = YAZI.plugin.fetchers.load_full();
		let mut loaded = self.scheduler.fetch.loaded.lock();
		let mut tasks: [Vec<_>; MAX_FETCHERS as usize] = Default::default();

		for f in paged {
			let hash = FileSig(f).hash_u64();
			for g in FetcherMatcher::new(&fetchers, f, mimetype.get(&f.url).unwrap_or_default()) {
				let fresh = match loaded.get_mut(&hash) {
					Some(l) => l.mark(g.idx, g.rev),
					None => loaded.put(hash, Loaded::new(g.idx, g.rev)).is_none(),
				};
				if fresh {
					tasks[g.idx as usize].push(f.clone());
				}
			}
		}

		drop(loaded);
		for (i, tasks) in tasks.into_iter().enumerate() {
			if !tasks.is_empty() {
				self.scheduler.fetch_paged(fetchers[i].clone(), tasks);
			}
		}
	}

	pub fn preload_paged(&self, paged: &[File], mimetype: &Mimetype) {
		let mut loaded = self.scheduler.preload.loaded.lock();
		for f in paged {
			let hash = FileSig(f).hash_u64();
			let mime = mimetype.get(&f.url).unwrap_or_default();
			for p in YAZI.plugin.preloaders.matches(f, mime) {
				let fresh = match loaded.get_mut(&hash) {
					Some(l) => l.mark(p.idx, p.rev),
					None => loaded.put(hash, Loaded::new(p.idx, p.rev)).is_none(),
				};
				if fresh {
					self.scheduler.preload_paged(p, f, mime.intern());
				}
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
					let key = f.key();
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
