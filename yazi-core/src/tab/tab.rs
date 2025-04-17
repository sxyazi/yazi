use std::iter;

use anyhow::Result;
use ratatui::layout::Rect;
use tokio::task::JoinHandle;
use yazi_adapter::Dimension;
use yazi_config::{LAYOUT, popup::{Origin, Position}};
use yazi_fs::{File, FolderStage};
use yazi_macro::render;
use yazi_shared::{Id, Ids, url::Url};

use super::{Backstack, Finder, Folder, History, Mode, Preference, Preview};
use crate::{spot::Spot, tab::Selected};

pub struct Tab {
	pub id:      Id,
	pub mode:    Mode,
	pub pref:    Preference,
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

impl Default for Tab {
	fn default() -> Self {
		static IDS: Ids = Ids::new();

		Self {
			id:      IDS.next(),
			mode:    Default::default(),
			pref:    Default::default(),
			current: Default::default(),
			parent:  Default::default(),

			backstack: Default::default(),
			history:   Default::default(),
			selected:  Default::default(),

			spot:    Default::default(),
			preview: Default::default(),
			finder:  Default::default(),
			search:  Default::default(),
		}
	}
}

impl Tab {
	pub fn shutdown(&mut self) {
		self.search.take().map(|h| h.abort());
		self.preview.reset();
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

		let mut rect = LAYOUT.get().current;
		rect.y = rect.y.saturating_sub(1) + y as u16;
		rect.height = 1;
		Some(rect)
	}

	pub fn hovered_rect_based(&self, pos: Position) -> Rect {
		let ws = Dimension::available().into();
		if let Some(r) = self.hovered_rect() {
			Position::sticky(ws, r, pos.offset)
		} else {
			Position::new(Origin::TopCenter, pos.offset).rect(ws)
		}
	}

	pub fn selected_or_hovered(&self) -> Box<dyn Iterator<Item = &Url> + '_> {
		if self.selected.is_empty() {
			Box::new(self.hovered().map(|h| vec![&h.url]).unwrap_or_default().into_iter())
		} else {
			Box::new(self.selected.keys())
		}
	}

	pub fn hovered_and_selected(&self) -> Box<dyn Iterator<Item = &Url> + '_> {
		let Some(h) = self.hovered() else { return Box::new(iter::empty()) };
		if self.selected.is_empty() {
			Box::new([&h.url, &h.url].into_iter())
		} else {
			Box::new([&h.url].into_iter().chain(self.selected.keys()))
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

			f.files.set_show_hidden(self.pref.show_hidden);
			f.files.set_sorter(<_>::from(&self.pref));

			render!(f.files.catchup_revision());
			render!(f.repos(f.trace.clone()));
		};

		apply(&mut self.current);

		if let Some(parent) = &mut self.parent {
			apply(parent);
			parent.hover(self.current.url.urn()); // The parent should always track the CWD
		}

		self
			.current
			.hovered()
			.filter(|h| h.is_dir())
			.and_then(|h| self.history.get_mut(&h.url))
			.map(apply);
	}
}
