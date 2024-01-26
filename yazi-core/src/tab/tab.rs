use std::{borrow::Cow, collections::BTreeMap};

use anyhow::Result;
use tokio::task::JoinHandle;
use yazi_shared::{fs::{File, Url}, render};

use super::{Backstack, Config, Finder, Mode, Preview};
use crate::folder::Folder;

pub struct Tab {
	pub mode:    Mode,
	pub conf:    Config,
	pub current: Folder,
	pub parent:  Option<Folder>,

	pub backstack: Backstack<Url>,
	pub history:   BTreeMap<Url, Folder>,

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
	// --- Mode
	#[inline]
	pub fn in_selecting(&self) -> bool { self.mode.is_visual() || self.current.files.has_selected() }

	// --- Current
	pub fn selected(&self) -> Vec<&File> {
		let pending = self.mode.visual().map(|(_, p)| Cow::Borrowed(p)).unwrap_or_default();
		let selected = self.current.files.selected(&pending, self.mode.is_unset());

		if selected.is_empty() {
			self.current.hovered().map(|h| vec![h]).unwrap_or_default()
		} else {
			selected
		}
	}

	// --- History
	#[inline]
	pub fn history(&self, url: &Url) -> Option<&Folder> { self.history.get(url) }

	#[inline]
	pub fn history_new(&mut self, url: &Url) -> Folder {
		self.history.remove(url).unwrap_or_else(|| Folder::from(url))
	}

	pub fn apply_files_attrs(&mut self) {
		let apply = |f: &mut Folder| {
			let hovered = f.hovered().filter(|_| f.tracing).map(|h| h.url());

			f.files.set_show_hidden(self.conf.show_hidden);
			f.files.set_sorter(self.conf.sorter());
			render!(f.files.catchup_revision());
			render!(f.repos(hovered));
		};

		apply(&mut self.current);

		if let Some(parent) = self.parent.as_mut() {
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
