use std::borrow::Cow;

use ratatui_core::layout::Rect;
use yazi_binding::position::{Origin, Position};
use yazi_config::LAYOUT;
use yazi_fs::file::File;
use yazi_shared::{id::{Id, Ids}, scope::{Scope, ScopeLike}, url::{UrlBuf, UrlLike}};
use yazi_term::TERM;

use super::{Backstack, Finder, Folder, History, Mode, Preference, Preview, PreviewMut};
use crate::{spot::{Spot, SpotMut}, tab::Selected};

pub struct Tab {
	pub id:      Id,
	pub scope:   Scope,
	pub mode:    Mode,
	pub pref:    Preference,
	pub current: Folder,
	pub parent:  Option<Folder>,

	pub backstack: Backstack,
	pub history:   History,
	pub selected:  Selected,

	pub spot:    Spot,
	pub preview: Preview,
	pub finder:  Option<Finder>,
}

impl Default for Tab {
	fn default() -> Self {
		static IDS: Ids = Ids::new();

		let id = IDS.next();
		let scope = Scope::new(id);

		Self {
			id,
			mode: Default::default(),
			pref: Default::default(),
			current: Default::default(),
			parent: Default::default(),

			backstack: Default::default(),
			history: Default::default(),
			selected: Default::default(),

			spot: Spot::new(&scope),
			preview: Preview::new(&scope),
			finder: Default::default(),

			scope,
		}
	}
}

impl Tab {
	pub fn shutdown(&mut self) {
		self.scope.cancel();
		self.preview_mut().reset();
	}
}

impl Tab {
	// --- Current
	#[inline]
	pub fn cwd(&self) -> &UrlBuf { &self.current.url }

	pub fn name(&self) -> Cow<'_, str> {
		let url = &self.current;
		if !self.pref.name.is_empty() {
			Cow::Borrowed(&self.pref.name)
		} else if let Some(s) = url.name() {
			s.to_string_lossy()
		} else {
			url.loc().to_string_lossy()
		}
	}

	#[inline]
	pub fn hovered(&self) -> Option<&File> { self.current.hovered() }

	#[inline]
	pub fn hovered_url(&self) -> Option<&UrlBuf> { self.current.hovered_url() }

	fn hovered_rect(&self) -> Option<Rect> {
		let y = self.current.entries.position(self.hovered()?.key())? - self.current.offset;

		let mut rect = LAYOUT.get().current;
		rect.y = rect.y.saturating_sub(1) + y as u16;
		rect.height = 1;
		Some(rect)
	}

	pub(crate) fn hovered_rect_based(&self, pos: Position) -> Rect {
		let area = TERM.dimension().area();
		if let Some(r) = self.hovered_rect() {
			pos.sticky(r, area)
		} else {
			Position::new(Origin::TopCenter, pos.offset).rect(area)
		}
	}

	pub fn selected_or_hovered_files(&self) -> impl Iterator<Item = &File> {
		self.selected.files().chain(self.hovered().filter(|_| self.selected.is_empty()))
	}

	pub fn selected_or_hovered_urls(&self) -> impl Iterator<Item = &UrlBuf> {
		self.selected_or_hovered_files().map(|f| &f.url)
	}

	// --- History
	#[inline]
	pub fn hovered_folder(&self) -> Option<&Folder> {
		self.hovered().filter(|&h| h.is_dir()).and_then(|h| self.history.get(&h.url))
	}

	#[inline]
	pub fn hovered_folder_mut(&mut self) -> Option<&mut Folder> {
		self.current.hovered_mut().filter(|h| h.is_dir()).and_then(|h| self.history.get_mut(&h.url))
	}

	// --- Spot
	#[inline]
	pub fn spot_mut(&mut self) -> SpotMut<'_> { self.into() }

	// --- Preview
	#[inline]
	pub fn preview_mut(&mut self) -> PreviewMut<'_> { self.into() }
}

impl ScopeLike for Tab {
	fn scope(&self) -> &Scope { &self.scope }
}
