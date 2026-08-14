use hashbrown::{HashMap, HashSet};
use yazi_fs::{FilesOp, file::File};
use yazi_shared::{path::{PathBufDyn, PathDyn}, url::{UrlBuf, UrlLike}};

use crate::{mgr::{Mgr, Yanked}, tab::Selected};

pub struct Reconciler<'a> {
	selected: &'a mut Selected,
	yanked:   &'a mut Yanked,
}

impl<'a> Reconciler<'a> {
	pub fn new(tab: usize, mgr: &'a mut Mgr) -> Self {
		let Mgr { tabs, yanked, .. } = mgr;
		Self { selected: &mut tabs[tab].selected, yanked }
	}

	pub fn apply(&mut self, op: &FilesOp) {
		match op {
			FilesOp::Full(_, files) => self.scan(op.cwd(), files, true),
			FilesOp::Part(_, files, _) | FilesOp::Creating(_, files) => {
				self.scan(op.cwd(), files, false);
			}
			FilesOp::Deleting(cwd, keys) => self.delete(cwd, keys),
			FilesOp::Updating(cwd, files) | FilesOp::Upserting(cwd, files) => {
				self.update(cwd, files);
			}
			_ => {}
		}
	}

	fn scan(&mut self, cwd: &UrlBuf, files: &[File], authoritative: bool) {
		let mut tracked: HashMap<_, _> = self
			.selected
			.urls()
			.chain(self.yanked.urls())
			.filter(|u| is_child(u, cwd))
			.map(|url| (url.key(), None))
			.collect();

		if tracked.is_empty() {
			return;
		}

		for file in files {
			tracked.get_mut(&file.key()).map(|f| *f = Some(file));
		}

		let selected = Patch::from_scan(self.selected.urls(), cwd, &tracked, authoritative);
		let yanked = Patch::from_scan(self.yanked.urls(), cwd, &tracked, authoritative);
		drop(tracked);

		selected.apply_selected(self.selected);
		yanked.apply_yanked(self.yanked);
	}

	fn delete(&mut self, cwd: &UrlBuf, keys: &HashSet<PathBufDyn>) {
		let selected = Patch::from_deleting(self.selected.urls(), cwd, keys);
		let yanked = Patch::from_deleting(self.yanked.urls(), cwd, keys);

		selected.apply_selected(self.selected);
		yanked.apply_yanked(self.yanked);
	}

	fn update(&mut self, cwd: &UrlBuf, files: &HashMap<PathBufDyn, File>) {
		let selected = Patch::from_updating(self.selected.urls(), cwd, files);
		let yanked = Patch::from_updating(self.yanked.urls(), cwd, files);

		selected.apply_selected(self.selected);
		yanked.apply_yanked(self.yanked);
	}
}

// --- Patch
#[derive(Default)]
struct Patch<'a> {
	removal: Vec<UrlBuf>,
	files:   Vec<&'a File>,
}

impl<'a> Patch<'a> {
	fn from_scan<'u>(
		urls: impl Iterator<Item = &'u UrlBuf>,
		cwd: &UrlBuf,
		tracked: &HashMap<PathDyn<'u>, Option<&'a File>>,
		authoritative: bool,
	) -> Self {
		let mut me = Self::default();
		for url in urls.filter(|u| is_child(u, cwd)) {
			match tracked.get(&url.key()).copied().flatten() {
				Some(file) if file.url == *url => me.files.push(file),
				Some(file) if authoritative => {
					me.removal.push(url.clone());
					me.files.push(file);
				}
				None if authoritative => me.removal.push(url.clone()),
				_ => {}
			}
		}
		me
	}

	fn from_deleting<'u>(
		urls: impl Iterator<Item = &'u UrlBuf>,
		cwd: &UrlBuf,
		keys: &HashSet<PathBufDyn>,
	) -> Self {
		Self {
			removal: urls
				.filter(|u| is_child(u, cwd))
				.filter(|u| keys.contains(&u.key()))
				.cloned()
				.collect(),
			files:   vec![],
		}
	}

	fn from_updating<'u>(
		urls: impl Iterator<Item = &'u UrlBuf>,
		cwd: &UrlBuf,
		files: &'a HashMap<PathBufDyn, File>,
	) -> Self {
		let mut me = Self::default();
		for url in urls.filter(|u| is_child(u, cwd)) {
			let Some(file) = files.get(&url.key()).filter(|f| !f.key().is_empty()) else {
				continue;
			};

			if file.url != *url {
				me.removal.push(url.clone());
			}
			me.files.push(file);
		}
		me
	}

	fn apply_selected(self, selected: &mut Selected) {
		selected.remove_many(&self.removal);
		for file in self.files {
			selected.upsert(file);
		}
	}

	fn apply_yanked(self, yanked: &mut Yanked) {
		yanked.remove_many(&self.removal);
		for file in self.files {
			yanked.upsert(file);
		}
	}
}

#[inline]
fn is_child(url: &UrlBuf, cwd: &UrlBuf) -> bool {
	url.pair().is_some_and(|(trail, _)| trail == *cwd)
}
