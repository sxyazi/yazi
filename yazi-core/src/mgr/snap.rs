use yazi_fs::Splatable;
use yazi_shared::url::{AsUrl, Url, UrlBuf};

use crate::{mgr::Mgr, tab::TabSnap};

pub struct MgrSnap {
	tab:    usize,
	tabs:   Vec<TabSnap>,
	yanked: Vec<UrlBuf>,
}

impl From<&Mgr> for MgrSnap {
	fn from(value: &Mgr) -> Self {
		Self {
			tab:    value.tabs.cursor,
			tabs:   value.tabs.iter().map(Into::into).collect(),
			yanked: value.yanked.urls().cloned().collect(),
		}
	}
}

impl Splatable for MgrSnap {
	fn tab(&self) -> usize { self.tab + 1 }

	fn selected(&self, tab: usize, mut idx: Option<usize>) -> impl Iterator<Item = Url<'_>> {
		idx = idx.and_then(|i| i.checked_sub(1));
		tab
			.checked_sub(1)
			.and_then(|tab| self.tabs.get(tab))
			.map_or_else(|| &[][..], |s| &s.selected)
			.iter()
			.skip(idx.unwrap_or(0))
			.take(if idx.is_some() { 1 } else { usize::MAX })
			.map(AsUrl::as_url)
	}

	fn hovered(&self, tab: usize) -> Option<Url<'_>> {
		tab
			.checked_sub(1)
			.and_then(|tab| self.tabs.get(tab))
			.and_then(|tab| tab.hovered.as_ref())
			.map(AsUrl::as_url)
	}

	fn yanked(&self, mut idx: Option<usize>) -> impl Iterator<Item = Url<'_>> {
		idx = idx.and_then(|i| i.checked_sub(1));
		self
			.yanked
			.iter()
			.skip(idx.unwrap_or(0))
			.take(if idx.is_some() { 1 } else { usize::MAX })
			.map(AsUrl::as_url)
	}
}
