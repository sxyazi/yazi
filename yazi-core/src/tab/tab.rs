use std::{collections::HashMap, iter};

use anyhow::Result;
use ratatui::layout::Rect;
use tokio::task::JoinHandle;
use yazi_adapter::Dimension;
use yazi_config::{popup::{Origin, Position}, LAYOUT};
use yazi_fs::{Folder, FolderStage};
use yazi_shared::{fs::Url, render};

use super::{Backstack, Config, Finder, Mode, Preview};
use crate::tab::Selected;

#[derive(Default)]
pub struct Tab {
	pub idx:     usize,
	pub mode:    Mode,
	pub conf:    Config,
	pub current: Folder,
	pub parent:  Option<Folder>,

	pub backstack: Backstack<Url>,
	pub history:   HashMap<Url, Folder>,
	pub selected:  Selected,

	pub preview:       Preview,
	pub finder:        Option<Finder>,
	pub(super) search: Option<JoinHandle<Result<()>>>,
}

impl Tab {
	pub fn shutdown(&mut self) {
		if let Some(handle) = self.search.take() {
			handle.abort();
		}
	}
}

impl Tab {
	// --- Current
	pub fn hovered_rect(&self) -> Option<Rect> {
		let y = self.current.files.position(&self.current.hovered()?.url)? - self.current.offset;

		let mut rect = LAYOUT.load().current;
		rect.y = rect.y.saturating_sub(1) + y as u16;
		rect.height = 1;
		Some(rect)
	}

	pub fn hovered_rect_based(&self, pos: Position) -> Rect {
		let ws = Dimension::available();
		if let Some(r) = self.hovered_rect() {
			Position::sticky(ws, r, pos.offset)
		} else {
			Position::new(Origin::TopCenter, pos.offset).rect(ws)
		}
	}

	pub fn selected_or_hovered(&self, reorder: bool) -> Box<dyn Iterator<Item = &Url> + '_> {
		if self.selected.is_empty() {
			Box::new(self.current.hovered().map(|h| vec![&h.url]).unwrap_or_default().into_iter())
		} else if !reorder {
			Box::new(self.selected.keys())
		} else {
			let mut vec: Vec<_> = self.selected.iter().collect();
			vec.sort_unstable_by(|a, b| a.1.cmp(b.1));
			Box::new(vec.into_iter().map(|(k, _)| k))
		}
	}

	pub fn hovered_and_selected(&self, reorder: bool) -> Box<dyn Iterator<Item = &Url> + '_> {
		let Some(h) = self.current.hovered() else { return Box::new(iter::empty()) };

		if self.selected.is_empty() {
			Box::new([&h.url, &h.url].into_iter())
		} else if !reorder {
			Box::new([&h.url].into_iter().chain(self.selected.keys()))
		} else {
			let mut vec: Vec<_> = self.selected.iter().collect();
			vec.sort_unstable_by(|a, b| a.1.cmp(b.1));
			Box::new([&h.url].into_iter().chain(vec.into_iter().map(|(k, _)| k)))
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
