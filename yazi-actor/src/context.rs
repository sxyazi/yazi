use std::{mem, ops::{Deref, DerefMut}};

use anyhow::{Result, anyhow};
use yazi_core::{Core, mgr::Tabs, tab::{Folder, Tab}};
use yazi_fs::file::File;
use yazi_shared::{Source, event::Action, id::Id, url::UrlBuf};
use yazi_tui::Raterm;

pub struct Ctx<'a> {
	pub(crate) core: &'a mut Core,
	pub(crate) term: &'a mut Option<Raterm>,
	pub(crate) tab:  usize,
	pub level:       usize,
	source:          Source,
	#[cfg(debug_assertions)]
	pub backtrace:   Vec<&'static str>,
}

impl Deref for Ctx<'_> {
	type Target = Core;

	fn deref(&self) -> &Self::Target { self.core }
}

impl DerefMut for Ctx<'_> {
	fn deref_mut(&mut self) -> &mut Self::Target { self.core }
}

impl<'a> Ctx<'a> {
	pub fn new(action: &Action, core: &'a mut Core, term: &'a mut Option<Raterm>) -> Result<Self> {
		let tab = if let Ok(id) = action.get::<Id>("tab") {
			core.mgr.tabs.idx(id).ok_or_else(|| anyhow!("Tab with id {id} not found"))?
		} else {
			core.mgr.tabs.cursor
		};

		Ok(Self {
			core,
			term,
			tab,
			level: 0,
			source: action.source,
			#[cfg(debug_assertions)]
			backtrace: vec![],
		})
	}

	pub(crate) fn with<F, T>(&mut self, tab: usize, f: F) -> T
	where
		F: FnOnce(&mut Self) -> T,
	{
		let prev = mem::replace(&mut self.tab, tab);
		let result = f(self);
		self.tab = prev;
		result
	}

	pub(crate) fn renew<'b>(cx: &'a mut Ctx<'b>) -> Self {
		let tab = cx.core.mgr.tabs.cursor;
		Self {
			core: cx.core,
			term: cx.term,
			tab,
			level: cx.level,
			source: cx.source,
			#[cfg(debug_assertions)]
			backtrace: vec![],
		}
	}

	pub fn active(core: &'a mut Core, term: &'a mut Option<Raterm>) -> Self {
		let tab = core.mgr.tabs.cursor;
		Self {
			core,
			term,
			tab,
			level: 0,
			source: Source::Unknown,
			#[cfg(debug_assertions)]
			backtrace: vec![],
		}
	}
}

impl<'a> Ctx<'a> {
	#[inline]
	pub(crate) fn tabs(&self) -> &Tabs { &self.mgr.tabs }

	#[inline]
	pub(crate) fn tabs_mut(&mut self) -> &mut Tabs { &mut self.mgr.tabs }

	#[inline]
	pub(crate) fn tab(&self) -> &Tab { &self.tabs()[self.tab] }

	#[inline]
	pub(crate) fn tab_mut(&mut self) -> &mut Tab { &mut self.core.mgr.tabs[self.tab] }

	#[inline]
	pub(crate) fn cwd(&self) -> &UrlBuf { self.tab().cwd() }

	#[inline]
	pub(crate) fn parent(&self) -> Option<&Folder> { self.tab().parent.as_ref() }

	#[inline]
	pub(crate) fn parent_mut(&mut self) -> Option<&mut Folder> { self.tab_mut().parent.as_mut() }

	#[inline]
	pub(crate) fn current(&self) -> &Folder { &self.tab().current }

	#[inline]
	pub(crate) fn current_mut(&mut self) -> &mut Folder { &mut self.tab_mut().current }

	#[inline]
	pub(crate) fn hovered(&self) -> Option<&File> { self.tab().hovered() }

	#[inline]
	pub(crate) fn hovered_url(&self) -> Option<&UrlBuf> { self.tab().hovered_url() }

	#[inline]
	pub(crate) fn hovered_folder(&self) -> Option<&Folder> { self.tab().hovered_folder() }

	#[inline]
	pub(crate) fn hovered_folder_mut(&mut self) -> Option<&mut Folder> {
		self.tab_mut().hovered_folder_mut()
	}

	pub(crate) fn source(&self) -> Source { if self.level != 1 { Source::Ind } else { self.source } }
}
