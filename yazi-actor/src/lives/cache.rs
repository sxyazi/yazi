use std::ops::Range;

use foldhash::fast::FixedState;
use hashbrown::{HashMap, hash_map::Entry};
use mlua::{AnyUserData, UserData};
use yazi_plugin::LUA;

use super::{File, Folder, MutCell, PtrCell, Tab};
use crate::mgr::MAX_TABS;

pub(super) static TABS: MutCell<[TabCache; MAX_TABS]> =
	MutCell::new([const { TabCache::new() }; MAX_TABS]);

pub(super) struct TabCache {
	ud:       Option<AnyUserData>,
	panes:    [PaneCache; 3],
	fallback: HashMap<PtrCell<yazi_fs::file::File>, AnyUserData, FixedState>,
}

struct PaneCache {
	folder: Option<PtrCell<yazi_core::tab::Folder>>,
	window: Range<usize>,
	files:  Vec<Option<AnyUserData>>,
}

impl TabCache {
	const fn new() -> Self {
		Self {
			ud:       None,
			panes:    [const { PaneCache { folder: None, window: 0..0, files: Vec::new() } }; 3],
			fallback: HashMap::with_hasher(FixedState::with_seed(0)),
		}
	}

	pub(super) fn get(idx: usize) -> &'static mut Self { unsafe { &mut (*TABS.get())[idx] } }

	pub(super) fn tab(&mut self, tab: Tab) -> mlua::Result<AnyUserData> {
		if self.ud.is_none() {
			for (pane, (folder, offset)) in self.panes.iter_mut().zip([
				(Some(&tab.current), tab.current.offset),
				(tab.parent.as_ref(), tab.parent.as_ref().map_or(0, |f| f.offset)),
				(tab.hovered_folder(), tab.preview.skip),
			]) {
				pane.folder = folder.map(Into::into);
				pane.window = folder.map(|f| Folder::window(offset, f)).unwrap_or_default();
			}
		}

		Self::userdata(&mut self.ud, tab)
	}

	pub(super) fn file(&mut self, file: File) -> mlua::Result<AnyUserData> {
		// Fast path: if the file is in a pane's visible window
		if let Some(pane) = self.pane(&file) {
			pane.files.resize_with(pane.window.len(), || None);
			return Self::userdata(&mut pane.files[file.idx - pane.window.start], file);
		}

		let ud = match self.fallback.entry(PtrCell(&*file)) {
			Entry::Occupied(e) => e.into_mut(),
			Entry::Vacant(e) => e.insert(LUA.create_userdata(file)?),
		};
		Ok(ud.clone())
	}

	fn pane(&mut self, file: &File) -> Option<&mut PaneCache> {
		self.panes.iter_mut().find(|p| p.folder == Some(file.folder) && p.window.contains(&file.idx))
	}

	pub(super) fn clear(&mut self) {
		let tab = self.ud.take();
		let files1 = self.panes.iter_mut().flat_map(|f| f.files.drain(..)).flatten();
		let files2 = self.fallback.drain().map(|(_, ud)| ud);

		for ud in files1.chain(files2).chain(tab) {
			ud.destroy().expect("failed to destruct scoped userdata");
		}
	}

	fn userdata<T: UserData + 'static>(
		slot: &mut Option<AnyUserData>,
		data: T,
	) -> mlua::Result<AnyUserData> {
		if let Some(ud) = slot {
			return Ok(ud.clone());
		}

		let ud = LUA.create_userdata(data)?;
		*slot = Some(ud.clone());
		Ok(ud)
	}
}
