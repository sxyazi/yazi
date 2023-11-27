use std::collections::{BTreeMap, HashMap, HashSet};

use yazi_shared::fs::Url;

use super::{Tabs, Watcher};
use crate::{files::{File, FilesOp}, tab::{Folder, Tab}, tasks::Tasks};

pub struct Manager {
	pub tabs:   Tabs,
	pub yanked: (bool, HashSet<Url>),

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

	pub fn update_read(&mut self, op: FilesOp) -> bool {
		let url = op.url().clone();
		let cwd = self.cwd().to_owned();
		let hovered = self.hovered().map(|h| h.url());

		let mut b = if cwd == url {
			self.current_mut().update(op)
		} else if matches!(self.parent(), Some(p) if p.cwd == url) {
			self.active_mut().parent.as_mut().unwrap().update(op)
		} else if matches!(self.hovered(), Some(h) if h.url == url) {
			self.active_mut().history.entry(url.clone()).or_insert_with(|| Folder::from(&url));
			self.active_mut().apply_files_attrs(true);
			self.active_mut().history.get_mut(&url).unwrap().update(op)
		} else {
			self.active_mut().history.entry(url.clone()).or_insert_with(|| Folder::from(&url)).update(op);
			false
		};

		b |= self.active_mut().parent.as_mut().is_some_and(|p| p.hover(&cwd));
		b |= hovered.as_ref().is_some_and(|h| self.current_mut().hover(h));

		if hovered.as_ref() != self.hovered().map(|h| &h.url) {
			Self::_hover(None);
		}
		b
	}

	pub fn update_ioerr(&mut self, op: FilesOp) -> bool {
		let url = op.url();
		let op = FilesOp::Full(url.clone(), Vec::new());

		if url == self.cwd() {
			self.current_mut().update(op);
			self.active_mut().leave(());
			true
		} else if matches!(self.parent(), Some(p) if &p.cwd == url) {
			self.active_mut().parent.as_mut().unwrap().update(op)
		} else {
			false
		}
	}

	pub fn update_mimetype(&mut self, mut mimes: BTreeMap<Url, String>, tasks: &Tasks) -> bool {
		mimes.retain(|f, m| self.mimetype.get(f) != Some(m));
		if mimes.is_empty() {
			return false;
		}

		tasks.precache_image(&mimes);
		tasks.precache_video(&mimes);
		tasks.precache_pdf(&mimes);

		self.mimetype.extend(mimes);
		true
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
	pub fn selected(&self) -> Vec<&File> { self.tabs.active().selected() }
}
