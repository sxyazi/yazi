use std::collections::BTreeMap;

use anyhow::Result;
use tokio::task::JoinHandle;
use yazi_shared::{fs::Url, render};

use super::{Backstack, Config, Finder, Mode, Preview};
use crate::{folder::{Folder, FolderStage}, tab::Selected};

pub struct Tab {
	pub mode:    Mode,
	pub conf:    Config,
	pub current: Folder,
	pub parent:  Option<Folder>,

	pub backstack: Backstack<Url>,
	pub history:   BTreeMap<Url, Folder>,
	pub selected:  Selected,

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
	pub fn selected_or_hovered(&self) -> Vec<&Url> {
		if self.selected.is_empty() {
			self.current.hovered().map(|h| vec![&h.url]).unwrap_or_default()
		} else {
			self.selected.iter().collect()
		}
	}

	pub fn hovered_and_selected(&self) -> Vec<&Url> {
		let Some(h) = self.current.hovered() else {
			return vec![];
		};

		if self.selected.is_empty() {
			vec![&h.url, &h.url]
		} else {
			[&h.url].into_iter().chain(self.selected.iter()).collect()
		}
	}

	// --- History
	#[inline]
	pub fn history_new(&mut self, url: &Url) -> Folder {
		self.history.remove(url).unwrap_or_else(|| Folder::from(url))
	}

	#[inline]
	pub fn hovered_folder(&self) -> Option<&Folder> {
		self.current.hovered().filter(|&h| h.is_dir()).and_then(|h| self.history.get(&h.url))
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
