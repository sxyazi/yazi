use ratatui_core::layout::Rect;
use yazi_binding::position::{Origin, Position};
use yazi_shared::url::UrlBuf;
use yazi_term::TERM;
use yazi_watcher::Watcher;

use super::{Batcher, Mimetype, Tabs, Yanked};
use crate::tab::{Folder, Tab};

pub struct Mgr {
	pub tabs:   Tabs,
	pub yanked: Yanked,

	pub batcher:  Batcher,
	pub watcher:  Watcher,
	pub mimetype: Mimetype,
}

impl Mgr {
	pub fn make() -> Self {
		Self {
			tabs:   Default::default(),
			yanked: Default::default(),

			batcher:  Default::default(),
			watcher:  Watcher::serve(),
			mimetype: Default::default(),
		}
	}

	pub fn area(&self, pos: Position) -> Rect {
		if pos.origin == Origin::Hovered {
			self.active().hovered_rect_based(pos)
		} else {
			pos.rect(TERM.dimension().area())
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
	pub fn current(&self) -> &Folder { &self.active().current }

	#[inline]
	pub fn current_mut(&mut self) -> &mut Folder { &mut self.active_mut().current }

	#[inline]
	pub fn parent_mut(&mut self) -> Option<&mut Folder> { self.active_mut().parent.as_mut() }
}
