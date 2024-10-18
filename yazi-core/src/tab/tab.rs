use std::iter;

use anyhow::Result;
use ratatui::layout::Rect;
use tokio::task::JoinHandle;
use yazi_adapter::Dimension;
use yazi_config::{LAYOUT, popup::{Origin, Position}};
use yazi_fs::{Folder, FolderStage};
use yazi_macro::render;
use yazi_shared::fs::{File, Url};

use super::{Backstack, Config, Finder, History, Mode, Preview, Spot};
use crate::tab::Selected;

#[derive(Default)]
pub struct Tab {
	pub idx:     usize,
	pub mode:    Mode,
	pub conf:    Config,
	pub current: Folder,
	pub parent:  Option<Folder>,

	pub backstack: Backstack<Url>,
	pub history:   History,
	pub selected:  Selected,

	pub spot:    Spot,
	pub preview: Preview,
	pub finder:  Option<Finder>,
	pub search:  Option<JoinHandle<Result<()>>>,
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
	#[inline]
	pub fn cwd(&self) -> &Url { &self.current.url }

	#[inline]
	pub fn hovered(&self) -> Option<&File> { self.current.hovered() }

	pub fn hovered_rect(&self) -> Option<Rect> {
		let y = self.current.files.position(self.hovered()?.urn())? - self.current.offset;

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
			Box::new(self.hovered().map(|h| vec![&h.url]).unwrap_or_default().into_iter())
		} else if !reorder {
			Box::new(self.selected.keys())
		} else {
			let mut vec: Vec<_> = self.selected.iter().collect();
			vec.sort_unstable_by(|a, b| a.1.cmp(b.1));
			Box::new(vec.into_iter().map(|(k, _)| k))
		}
	}

	pub fn hovered_and_selected(&self, reorder: bool) -> Box<dyn Iterator<Item = &Url> + '_> {
		let Some(h) = self.hovered() else { return Box::new(iter::empty()) };

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
	pub fn hovered_folder(&self) -> Option<&Folder> {
		self.hovered().filter(|&h| h.is_dir()).and_then(|h| self.history.get(&h.url))
	}

	pub fn apply_files_attrs(&mut self) {
		let apply = |f: &mut Folder| {
			if f.stage == FolderStage::Loading {
				return render!();
			}

			let hovered = f.hovered().filter(|_| f.tracing).map(|h| h.urn_owned());
			f.files.set_show_hidden(self.conf.show_hidden);
			f.files.set_sorter(self.conf.sorter());

			render!(f.files.catchup_revision());
			render!(f.repos(hovered.as_ref().map(|u| u.as_urn())));
		};

		apply(&mut self.current);

		if let Some(parent) = &mut self.parent {
			apply(parent);

			// The parent should always track the CWD
			parent.hover(self.current.url.urn());
			parent.tracing = parent.hovered().map(|h| &h.url) == Some(&self.current.url);
		}

		self
			.current
			.hovered()
			.filter(|h| h.is_dir())
			.and_then(|h| self.history.get_mut(&h.url))
			.map(apply);
	}
}
