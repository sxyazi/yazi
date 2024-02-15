use std::collections::HashMap;

use yazi_shared::fs::{File, Url};

use super::{Tabs, Watcher, Yanked};
use crate::{folder::Folder, tab::Tab};

pub struct Manager {
	pub tabs:   Tabs,
	pub yanked: Yanked,

	pub(super) watcher: Watcher,
	pub mimetype:       HashMap<Url, String>,
}

impl Manager {
	pub fn make() -> Self {
		Self {
			tabs:   Tabs::make(),
			yanked: Default::default(),

			watcher:  Watcher::start(),
			mimetype: Default::default(),
		}
	}
}

impl Manager {
	#[inline]
	pub fn cwd(&self) -> &Url { &self.current().cwd }

	#[inline]
	pub fn active(&self) -> &Tab { self.tabs.active() }

	#[inline]
	pub fn active_mut(&mut self) -> &mut Tab { self.tabs.active_mut() }

	#[inline]
	pub fn current(&self) -> &Folder { &self.tabs.active().current }

	#[inline]
	pub fn current_mut(&mut self) -> &mut Folder { &mut self.tabs.active_mut().current }

	#[inline]
	pub fn parent(&self) -> Option<&Folder> { self.tabs.active().parent.as_ref() }

	#[inline]
	pub fn hovered(&self) -> Option<&File> { self.tabs.active().current.hovered() }

	#[inline]
	pub fn hovered_folder(&self) -> Option<&Folder> { self.tabs.active().hovered_folder() }

	#[inline]
	pub fn selected_or_hovered(&self) -> Vec<&Url> { self.tabs.active().selected_or_hovered() }
}
