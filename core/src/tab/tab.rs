use std::{borrow::Cow, collections::BTreeMap};

use anyhow::Result;
use config::MANAGER;
use shared::Url;
use tokio::task::JoinHandle;

use super::{Backstack, Finder, Folder, Mode};
use crate::{emit, files::{File, FilesSorter}, preview::{Preview, PreviewLock}};

pub struct Tab {
	pub mode:    Mode,
	pub current: Folder,
	pub parent:  Option<Folder>,

	pub backstack: Backstack<Url>,
	pub history:   BTreeMap<Url, Folder>,
	pub preview:   Preview,

	pub finder:        Option<Finder>,
	pub(super) search: Option<JoinHandle<Result<()>>>,
	pub sorter:        FilesSorter,
	pub show_hidden:   bool,
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
			sorter: Default::default(),
			show_hidden: MANAGER.show_hidden,
		}
	}
}

impl From<&Url> for Tab {
	fn from(url: &Url) -> Self { Self::from(url.clone()) }
}

impl Tab {
	pub fn update_peek(&mut self, max: usize, url: Url) -> bool {
		let Some(hovered) = self.current.hovered() else {
			return false;
		};

		if url != hovered.url {
			return false;
		}

		self.preview.arrow_max(max)
	}

	pub fn update_preview(&mut self, lock: PreviewLock) -> bool {
		let Some(hovered) = self.current.hovered().map(|h| &h.url) else {
			return self.preview_reset();
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

	// --- Preview
	#[inline]
	pub fn preview_reset(&mut self) -> bool { self.preview.reset(|_| true) }

	#[inline]
	pub fn preview_reset_image(&mut self) -> bool { self.preview.reset(|l| l.is_image()) }

	#[inline]
	pub fn preview_arrow(&mut self, step: isize) -> bool { self.preview.arrow(step) }

	// --- Sorter
	pub fn set_sorter(&mut self, sorter: FilesSorter) -> bool {
		if sorter == self.sorter {
			return false;
		}

		self.sorter = sorter;
		self.apply_files_attrs(false)
	}

	// --- Show hidden
	pub fn set_show_hidden(&mut self, state: Option<bool>) -> bool {
		let state = state.unwrap_or(!self.show_hidden);
		if state == self.show_hidden {
			return false;
		}

		self.show_hidden = state;
		if self.apply_files_attrs(false) {
			emit!(Peek);
			return true;
		}
		false
	}

	pub fn apply_files_attrs(&mut self, just_preview: bool) -> bool {
		let mut b = false;
		if let Some(f) =
			self.current.hovered().filter(|h| h.is_dir()).and_then(|h| self.history.get_mut(&h.url))
		{
			b |= f.files.set_show_hidden(self.show_hidden);
			b |= f.files.set_sorter(self.sorter);
		}

		if just_preview {
			return b;
		}

		let hovered = self.current.hovered().map(|h| h.url());
		b |= self.current.files.set_show_hidden(self.show_hidden);
		b |= self.current.files.set_sorter(self.sorter);

		if let Some(parent) = self.parent.as_mut() {
			b |= parent.files.set_show_hidden(self.show_hidden);
			b |= parent.files.set_sorter(self.sorter);
		}

		self.current.repos(hovered);
		b
	}
}
