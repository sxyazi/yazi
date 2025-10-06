use std::iter;

use ratatui::layout::Rect;
use yazi_adapter::Dimension;
use yazi_config::popup::{Origin, Position};
use yazi_fs::Splatable;
use yazi_shared::url::{AsUrl, Url, UrlBuf};
use yazi_watcher::Watcher;

use super::{Mimetype, Tabs, Yanked};
use crate::tab::{Folder, Tab};

pub struct Mgr {
	pub tabs:   Tabs,
	pub yanked: Yanked,

	pub watcher:  Watcher,
	pub mimetype: Mimetype,
}

impl Mgr {
	pub fn make() -> Self {
		Self {
			tabs:   Default::default(),
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
	pub fn cwd(&self) -> &UrlBuf { self.active().cwd() }

	#[inline]
	pub fn active(&self) -> &Tab { self.tabs.active() }

	#[inline]
	pub fn active_mut(&mut self) -> &mut Tab { self.tabs.active_mut() }

	#[inline]
	pub fn current_mut(&mut self) -> &mut Folder { &mut self.active_mut().current }

	#[inline]
	pub fn parent_mut(&mut self) -> Option<&mut Folder> { self.active_mut().parent.as_mut() }
}

impl Splatable for Mgr {
	fn tab(&self) -> usize { self.tabs.cursor }

	fn selected(&self, tab: usize, mut idx: Option<usize>) -> impl Iterator<Item = Url<'_>> {
		idx = idx.and_then(|i| i.checked_sub(1));
		tab
			.checked_sub(1)
			.and_then(|tab| self.tabs.get(tab))
			.map(|tab| tab.selected_or_hovered())
			.unwrap_or_else(|| Box::new(iter::empty()))
			.skip(idx.unwrap_or(0))
			.take(if idx.is_some() { 1 } else { usize::MAX })
			.map(|u| u.as_url())
	}

	fn hovered(&self, tab: usize) -> Option<Url<'_>> {
		tab
			.checked_sub(1)
			.and_then(|tab| self.tabs.get(tab))
			.and_then(|tab| tab.hovered())
			.map(|h| h.url.as_url())
	}

	fn yanked(&self) -> impl Iterator<Item = Url<'_>> { self.yanked.iter().map(|u| u.as_url()) }
}
