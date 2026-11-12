use yazi_fs::{file::{File, FileRef}, files::{FileIter, Files}, splatter::Splatable};

use crate::{mgr::Mgr, tab::TabSnap};

pub struct MgrSnap {
	tab:    usize,
	tabs:   Vec<TabSnap>,
	yanked: Files,
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

	fn selected(&self, tab: usize) -> mlua::Result<FileIter<impl Iterator<Item = &File>>> {
		let files = match tab.checked_sub(1).and_then(|tab| self.tabs.get(tab)) {
			Some(s) if !s.selected.is_empty() => &s.selected,
			Some(s) => s.hovered.as_slice(),
			None => &[],
		};
		Ok(FileIter::new(files.iter()))
	}

	fn hovered(&self, tab: usize) -> mlua::Result<Option<FileRef<'_>>> {
		Ok(
			tab
				.checked_sub(1)
				.and_then(|tab| self.tabs.get(tab))
				.and_then(|tab| tab.hovered.as_ref())
				.map(Into::into),
		)
	}

	fn yanked(&self) -> mlua::Result<FileIter<impl Iterator<Item = &File>>> {
		Ok(FileIter::new(self.yanked.iter()))
	}
}
