use std::ops::{Deref, DerefMut};

use anyhow::{Result, anyhow};
use yazi_core::{Core, mgr::Tabs, tab::{Folder, Tab}, tasks::Tasks};
use yazi_fs::File;
use yazi_shared::{Id, Source, event::Action, url::UrlBuf};
use yazi_term::Term;

pub struct Ctx<'a> {
	pub core:      &'a mut Core,
	pub term:      &'a mut Option<Term>,
	pub tab:       usize,
	pub level:     usize,
	pub source:    Source,
	#[cfg(debug_assertions)]
	pub backtrace: Vec<&'static str>,
}

impl Deref for Ctx<'_> {
	type Target = Core;

	fn deref(&self) -> &Self::Target { self.core }
}

impl DerefMut for Ctx<'_> {
	fn deref_mut(&mut self) -> &mut Self::Target { self.core }
}

impl<'a> Ctx<'a> {
	#[inline]
	pub fn new(action: &Action, core: &'a mut Core, term: &'a mut Option<Term>) -> Result<Self> {
		let tab = if let Ok(id) = action.get::<Id>("tab") {
			core
				.mgr
				.tabs
				.iter()
				.position(|t| t.id == id)
				.ok_or_else(|| anyhow!("Tab with id {id} not found"))?
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

	#[inline]
	pub fn renew<'b>(cx: &'a mut Ctx<'b>) -> Self {
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

	#[inline]
	pub fn active(core: &'a mut Core, term: &'a mut Option<Term>) -> Self {
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
	pub fn tabs(&self) -> &Tabs { &self.mgr.tabs }

	#[inline]
	pub fn tabs_mut(&mut self) -> &mut Tabs { &mut self.mgr.tabs }

	#[inline]
	pub fn tab(&self) -> &Tab { &self.tabs()[self.tab] }

	#[inline]
	pub fn tab_mut(&mut self) -> &mut Tab { &mut self.core.mgr.tabs[self.tab] }

	#[inline]
	pub fn cwd(&self) -> &UrlBuf { self.tab().cwd() }

	#[inline]
	pub fn parent(&self) -> Option<&Folder> { self.tab().parent.as_ref() }

	#[inline]
	pub fn parent_mut(&mut self) -> Option<&mut Folder> { self.tab_mut().parent.as_mut() }

	#[inline]
	pub fn current(&self) -> &Folder { &self.tab().current }

	#[inline]
	pub fn current_mut(&mut self) -> &mut Folder { &mut self.tab_mut().current }

	#[inline]
	pub fn hovered(&self) -> Option<&File> { self.tab().hovered() }

	#[inline]
	pub fn hovered_folder(&self) -> Option<&Folder> { self.tab().hovered_folder() }

	#[inline]
	pub fn hovered_folder_mut(&mut self) -> Option<&mut Folder> {
		self.tab_mut().hovered_folder_mut()
	}

	#[inline]
	pub fn tasks(&self) -> &Tasks { &self.tasks }

	#[inline]
	pub fn source(&self) -> Source { if self.level != 1 { Source::Ind } else { self.source } }
}
