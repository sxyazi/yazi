use std::{mem, ops::{Deref, DerefMut}};

use anyhow::{Result, anyhow};
use yazi_fs::file::File;
use yazi_shared::{Source, event::Action, id::Id, scope::{Scope, ScopeLike}, url::UrlBuf};

use crate::{Core, mgr::Tabs, spot::{Spot, SpotMut}, tab::{Folder, Preview, PreviewMut, Tab}};

pub struct Ctx<'a> {
	pub core:      &'a mut Core,
	pub scope:     Scope,
	pub tab:       usize,
	pub level:     usize,
	source:        Source,
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

impl ScopeLike for Ctx<'_> {
	fn scope(&self) -> &Scope { &self.scope }
}

impl<'a> Ctx<'a> {
	pub fn new(action: &Action, core: &'a mut Core) -> Result<Self> {
		let mut scope = action.scope.clone();
		let tab = if let Some(id) = scope.tab.nz() {
			core.mgr.tabs.idx(id).ok_or_else(|| anyhow!("Tab with id {id} not found"))?
		} else {
			let tab = core.mgr.tabs.cursor;
			scope.tab = core.mgr.tabs[tab].id;
			tab
		};

		Ok(Self { scope, tab, source: action.source, ..Self::active(core) })
	}

	pub fn with<F, T>(&mut self, tab: usize, f: F) -> T
	where
		F: FnOnce(&mut Self) -> T,
	{
		let id = self.mgr.tabs[tab].id;
		let prev_id = mem::replace(&mut self.scope.tab, id);
		let prev = mem::replace(&mut self.tab, tab);

		let result = f(self);

		self.tab = prev;
		self.scope.tab = prev_id;
		result
	}

	pub fn renew(cx: &'a mut Ctx) -> Self {
		Self { level: cx.level, source: cx.source, ..Self::active(cx.core) }
	}

	pub fn active(core: &'a mut Core) -> Self {
		let tab = core.mgr.tabs.cursor;
		let scope = Scope::default().with_tab(core.mgr.tabs[tab].id);
		Self {
			scope,
			core,
			tab,
			level: 0,
			source: Source::Unknown,
			#[cfg(debug_assertions)]
			backtrace: vec![],
		}
	}

	pub fn nested(&mut self, action: &Action) -> Result<Ctx<'_>> {
		let mut cx = Ctx::new(action, self.core)?;

		cx.level = self.level;
		#[cfg(debug_assertions)]
		cx.backtrace.clone_from(&self.backtrace);

		Ok(cx)
	}

	pub fn indices_or_tab(&self, ids: Vec<Id>) -> Vec<usize> {
		if ids.is_empty() {
			vec![self.tab]
		} else {
			ids.into_iter().filter_map(|id| self.tabs().idx(id)).collect()
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
	pub fn hovered_url(&self) -> Option<&UrlBuf> { self.tab().hovered_url() }

	#[inline]
	pub fn hovered_folder(&self) -> Option<&Folder> { self.tab().hovered_folder() }

	#[inline]
	pub fn hovered_folder_mut(&mut self) -> Option<&mut Folder> {
		self.tab_mut().hovered_folder_mut()
	}

	#[inline]
	pub fn spot(&self) -> &Spot { &self.tab().spot }

	#[inline]
	pub fn spot_mut(&mut self) -> SpotMut<'_> { self.tab_mut().spot_mut() }

	#[inline]
	pub fn preview(&self) -> &Preview { &self.tab().preview }

	#[inline]
	pub fn preview_mut(&mut self) -> PreviewMut<'_> { self.tab_mut().preview_mut() }

	#[inline]
	pub fn source(&self) -> Source { if self.level == 1 { self.source } else { Source::Ind } }
}
