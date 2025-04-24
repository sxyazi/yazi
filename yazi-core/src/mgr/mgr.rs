use ratatui::layout::Rect;
use yazi_adapter::Dimension;
use yazi_config::popup::{Origin, Position};
use yazi_fs::File;
use yazi_shared::url::Url;

use super::{Mimetype, Tabs, Watcher, Yanked};
use crate::tab::{Folder, Tab};

pub struct Mgr {
	pub tabs:   Tabs,
	pub yanked: Yanked,

	pub(super) watcher: Watcher,
	pub mimetype:       Mimetype,
}

impl Mgr {
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
			pos.rect(Dimension::available().into())
		}
	}

	pub fn shutdown(&mut self) { self.tabs.iter_mut().for_each(|t| t.shutdown()); }
}

impl Mgr {
	#[inline]
	pub fn cwd(&self) -> &Url { self.active().cwd() }

	#[inline]
	pub fn active(&self) -> &Tab { self.tabs.active() }

	#[inline]
	pub fn active_mut(&mut self) -> &mut Tab { self.tabs.active_mut() }

	#[inline]
	pub fn current(&self) -> &Folder { &self.active().current }

	#[inline]
	pub fn current_mut(&mut self) -> &mut Folder { &mut self.active_mut().current }

	#[inline]
	pub fn parent(&self) -> Option<&Folder> { self.active().parent.as_ref() }

	#[inline]
	pub fn parent_mut(&mut self) -> Option<&mut Folder> { self.active_mut().parent.as_mut() }

	#[inline]
	pub fn hovered(&self) -> Option<&File> { self.active().hovered() }

	#[inline]
	pub fn hovered_folder(&self) -> Option<&Folder> { self.active().hovered_folder() }

	#[inline]
	pub fn selected_or_hovered(&self) -> Box<dyn Iterator<Item = &Url> + '_> {
		self.tabs.active().selected_or_hovered()
	}

	#[inline]
	pub fn hovered_and_selected(&self) -> Box<dyn Iterator<Item = &Url> + '_> {
		self.tabs.active().hovered_and_selected()
	}
}
