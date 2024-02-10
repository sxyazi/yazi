use std::{borrow::Cow, collections::{BTreeMap, BTreeSet}};

use anyhow::Result;
use tokio::task::JoinHandle;
use yazi_shared::{fs::{File, Url}, render};

use super::{Backstack, Config, Finder, Mode, Preview};
use crate::folder::{Folder, FolderStage};

pub struct Tab {
	pub mode:    Mode,
	pub conf:    Config,
	pub current: Folder,
	pub parent:  Option<Folder>,

	pub backstack: Backstack<Url>,
	pub history:   BTreeMap<Url, Folder>,
	pub selected:  BTreeSet<Url>,

	pub preview:       Preview,
	pub finder:        Option<Finder>,
	pub(super) search: Option<JoinHandle<Result<()>>>,
}

impl From<Url> for Tab {
	fn from(url: Url) -> Self {
		let parent = url.parent_url().map(Folder::from);

		Self {
			mode: Default::default(),
			current: Folder::from(url.clone()),
			parent,

			backstack: Backstack::new(url),
			history: Default::default(),
			selected: Default::default(),

			preview: Default::default(),
			finder: None,
			search: None,

			conf: Default::default(),
		}
	}
}

impl From<&Url> for Tab {
	fn from(url: &Url) -> Self { Self::from(url.clone()) }
}

impl Tab {
	// --- Current
	#[inline]
	pub fn in_selecting(&self) -> bool {
		!self.selected.is_empty() || self.mode.visual().is_some_and(|(_, indices)| !indices.is_empty())
	}

	pub fn selected(&self) -> Vec<&File> {
		let pending = self.mode.visual().map(|(_, p)| Cow::Borrowed(p)).unwrap_or_default();
		let is_unset = self.mode.is_unset();
		if self.selected.is_empty() && (is_unset || pending.is_empty()) {
			return vec![];
		}

		let selected: BTreeSet<_> = self.selected.iter().collect();
		let pending: BTreeSet<_> =
			pending.iter().filter_map(|&i| self.current.files.get(i)).map(|f| &f.url).collect();

		let urls: BTreeSet<_> = if is_unset {
			selected.difference(&pending).copied().collect()
		} else {
			selected.union(&pending).copied().collect()
		};

		let mut items = Vec::with_capacity(urls.len());
		for item in self.current.files.iter() {
			if urls.contains(&item.url) {
				items.push(item);
				if items.len() == urls.len() {
					break;
				}
			}
		}
		items
	}

	// --- History
	#[inline]
	pub fn history_new(&mut self, url: &Url) -> Folder {
		self.history.remove(url).unwrap_or_else(|| Folder::from(url))
	}

	pub fn apply_files_attrs(&mut self) {
		let apply = |f: &mut Folder| {
			if f.stage == FolderStage::Loading {
				return render!();
			}

			let hovered = f.hovered().filter(|_| f.tracing).map(|h| h.url());
			f.files.set_show_hidden(self.conf.show_hidden);
			f.files.set_sorter(self.conf.sorter());

			render!(f.files.catchup_revision());
			render!(f.repos(hovered));
		};

		apply(&mut self.current);

		if let Some(parent) = &mut self.parent {
			apply(parent);

			// The parent should always track the CWD
			parent.hover(&self.current.cwd);
			parent.tracing = parent.hovered().map(|h| &h.url) == Some(&self.current.cwd);
		}

		self
			.current
			.hovered()
			.filter(|h| h.is_dir())
			.and_then(|h| self.history.get_mut(&h.url))
			.map(apply);
	}
}
