use ratatui::layout::Rect;
use yazi_adapter::Dimension;
use yazi_config::popup::{Origin, Position};
use yazi_fs::Folder;
use yazi_shared::fs::{File, Url};

use super::{Mimetype, Tabs, Watcher, Yanked};
use crate::tab::Tab;

pub struct Manager {
	pub tabs:   Tabs,
	pub yanked: Yanked,

	pub(super) watcher: Watcher,
	pub mimetype:       Mimetype,
}

impl Manager {
	pub fn make() -> Self {
		Self {
			tabs:   Tabs::make(),
			yanked: Default::default(),

			watcher:  Watcher::serve(),
			mimetype: Default::default(),
		}
	}

	pub fn area(&self, pos: Position) -> Rect {
		if pos.origin == Origin::Hovered {
			self.active().hovered_rect_based(pos)
		} else {
			pos.rect(Dimension::available())
		}
	}

	pub fn shutdown(&mut self) { self.tabs.iter_mut().for_each(|t| t.shutdown()); }
}

impl Manager {
	#[inline]
	pub fn cwd(&self) -> &Url { &self.current().loc }

	#[inline]
	pub fn active(&self) -> &Tab { self.tabs.active() }

	#[inline]
	pub fn active_mut(&mut self) -> &mut Tab { self.tabs.active_mut() }

	#[inline]
	pub fn active_or(&self, idx: Option<usize>) -> &Tab { self.tabs.active_or(idx) }

	#[inline]
	pub fn active_or_mut(&mut self, idx: Option<usize>) -> &mut Tab { self.tabs.active_or_mut(idx) }

	#[inline]
	pub fn current(&self) -> &Folder { &self.active().current }

	#[inline]
	pub fn current_mut(&mut self) -> &mut Folder { &mut self.active_mut().current }

	#[inline]
	pub fn current_or(&self, idx: Option<usize>) -> &Folder { &self.active_or(idx).current }

	#[inline]
	pub fn current_or_mut(&mut self, idx: Option<usize>) -> &mut Folder {
		&mut self.active_or_mut(idx).current
	}

	#[inline]
	pub fn parent(&self) -> Option<&Folder> { self.active().parent.as_ref() }

	#[inline]
	pub fn hovered(&self) -> Option<&File> { self.active().current.hovered() }

	#[inline]
	pub fn hovered_folder(&self) -> Option<&Folder> { self.active().hovered_folder() }

	#[inline]
	pub fn selected_or_hovered(&self, reorder: bool) -> Box<dyn Iterator<Item = &Url> + '_> {
		self.tabs.active().selected_or_hovered(reorder)
	}

	#[inline]
	pub fn hovered_and_selected(&self, reorder: bool) -> Box<dyn Iterator<Item = &Url> + '_> {
		self.tabs.active().hovered_and_selected(reorder)
	}
}
