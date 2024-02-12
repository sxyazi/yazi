use std::ops::{Deref, Range};

use mlua::{AnyUserData, Lua, MetaMethod, UserDataMethods};

use super::{File, SCOPE};

pub(super) struct Files {
	window: Range<usize>,
	folder: *const yazi_core::folder::Folder,
	tab:    *const yazi_core::tab::Tab,
}

impl Deref for Files {
	type Target = yazi_core::folder::Files;

	fn deref(&self) -> &Self::Target { &self.folder().files }
}

impl Files {
	#[inline]
	pub(super) fn make(
		window: Range<usize>,
		folder: &yazi_core::folder::Folder,
		tab: &yazi_core::tab::Tab,
	) -> mlua::Result<AnyUserData<'static>> {
		SCOPE.create_any_userdata(Self { window, folder, tab })
	}

	pub(super) fn register(lua: &Lua) -> mlua::Result<()> {
		lua.register_userdata_type::<Self>(|reg| {
			reg.add_meta_method(MetaMethod::Len, |_, me, ()| Ok(me.window.end - me.window.start));

			reg.add_meta_method(MetaMethod::Index, |_, me, mut idx: usize| {
				idx += me.window.start;
				if idx > me.window.end || idx == 0 {
					Ok(None)
				} else {
					Some(File::make(idx - 1, me.folder(), me.tab())).transpose()
				}
			});
		})?;

		Ok(())
	}

	#[inline]
	fn folder(&self) -> &yazi_core::folder::Folder { unsafe { &*self.folder } }

	#[inline]
	fn tab(&self) -> &yazi_core::tab::Tab { unsafe { &*self.tab } }
}
