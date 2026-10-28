use yazi_config::{YAZI, plugin::{FetcherMatcher, MAX_FETCHERS}};
use yazi_fs::{Entries, FsHash64, SortBy, file::{File, FileSig, Files}};
use yazi_scheduler::{Loaded, fetch::FetchInFetch, preload::PreloadInPreload};

use super::Tasks;
use crate::{mgr::Mimetype, tab::Tab};

impl Tasks {
	pub fn fetch_paged(&self, tab: &Tab, paged: &[File], mimetype: &Mimetype) {
		let fetchers = YAZI.plugin.fetchers.load_full();
		let mut loaded = self.scheduler.fetch.loaded.lock();
		let mut tasks: [(Files, Vec<_>); MAX_FETCHERS as usize] = Default::default();

		for f in paged {
			let hash = FileSig(f).hash_u64();
			for g in FetcherMatcher::new(&fetchers, f, mimetype.get(&f.url).unwrap_or_default()) {
				if loaded.get_or_insert_mut(hash, Loaded::default).mark(g.idx, g.rev) {
					let (targets, hashes) = &mut tasks[g.idx as usize];
					targets.push(f.clone());
					hashes.push(hash);
				}
			}
		}

		drop(loaded);

		for (i, (targets, hashes)) in tasks.into_iter().enumerate() {
			if !targets.is_empty() {
				self.scheduler.fetch_paged(FetchInFetch::hashed(&fetchers[i], targets, hashes).scoped(tab));
			}
		}
	}

	pub fn preload_paged(&self, tab: &Tab, paged: &[File], mimetype: &Mimetype) {
		let mut loaded = self.scheduler.preload.loaded.lock();
		for f in paged {
			let hash = FileSig(f).hash_u64();
			let mime = mimetype.get(&f.url).unwrap_or_default();
			for p in YAZI.plugin.preloaders.matches(f, mime) {
				if loaded.get_or_insert_mut(hash, Loaded::default).mark(p.idx, p.rev) {
					self.scheduler.preload_paged(PreloadInPreload::new(p, f, hash, mime).scoped(tab));
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
