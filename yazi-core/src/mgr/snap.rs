use yazi_fs::{Splatable, file::File};

use crate::{mgr::Mgr, tab::TabSnap};

pub struct MgrSnap {
	tab:    usize,
	tabs:   Vec<TabSnap>,
	yanked: Vec<File>,
}

impl From<&Mgr> for MgrSnap {
	fn from(value: &Mgr) -> Self {
		Self {
			tab:    value.tabs.cursor,
			tabs:   value.tabs.iter().map(Into::into).collect(),
			yanked: value.yanked.files().cloned().collect(),
		}
	}
}

impl Splatable for MgrSnap {
	fn tab(&self) -> usize { self.tab + 1 }

	fn selected(&self, tab: usize, mut idx: Option<usize>) -> impl Iterator<Item = &File> {
		idx = idx.and_then(|i| i.checked_sub(1));
		tab
			.checked_sub(1)
			.and_then(|tab| self.tabs.get(tab))
			.map_or_else(|| &[][..], |s| &s.selected)
			.iter()
			.skip(idx.unwrap_or(0))
			.take(if idx.is_some() { 1 } else { usize::MAX })
	}

	fn hovered(&self, tab: usize) -> Option<&File> {
		tab.checked_sub(1).and_then(|tab| self.tabs.get(tab)).and_then(|tab| tab.hovered.as_ref())
	}

	fn yanked(&self, mut idx: Option<usize>) -> impl Iterator<Item = &File> {
		idx = idx.and_then(|i| i.checked_sub(1));
		self.yanked.iter().skip(idx.unwrap_or(0)).take(if idx.is_some() { 1 } else { usize::MAX })
	}
}
