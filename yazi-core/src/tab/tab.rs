use std::{borrow::Cow, collections::BTreeMap};

use anyhow::Result;
use tokio::task::JoinHandle;
use yazi_shared::fs::Url;

use super::{Backstack, Config, Finder, Folder, Mode};
use crate::{files::File, preview::{Preview, PreviewLock}};

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
	pub fn update_preview(&mut self, lock: PreviewLock) -> bool {
		let Some(hovered) = self.current.hovered().map(|h| &h.url) else {
			return self.preview.reset();
		};

		if lock.url != *hovered {
			return false;
		}

		self.preview.lock = Some(lock);
		true
	}
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

	pub fn apply_files_attrs(&mut self, just_preview: bool) -> bool {
		let apply = |f: &mut Folder| {
			let hovered = f.hovered().map(|h| h.url());

			let mut b = f.files.set_show_hidden(self.conf.show_hidden);
			b |= f.files.set_sorter(self.conf.sorter());
			b | f.repos(hovered)
		};

		let mut b = false;
		if let Some(f) =
			self.current.hovered().filter(|h| h.is_dir()).and_then(|h| self.history.get_mut(&h.url))
		{
			b |= apply(f);
		}
		if just_preview {
			return b;
		}

		b |= apply(&mut self.current);
		if let Some(parent) = self.parent.as_mut() {
			b |= apply(parent);
		}

		b
	}
}
