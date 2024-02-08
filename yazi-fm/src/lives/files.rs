use std::ops::{Deref, Range};

use mlua::{Lua, MetaMethod, UserDataMethods};

use super::File;

pub(super) struct Files {
	folder: *const yazi_core::folder::Folder,
	window: Range<usize>,
}

impl Deref for Files {
	type Target = yazi_core::folder::Files;

	fn deref(&self) -> &Self::Target { &self.folder().files }
}

impl Files {
	#[inline]
	pub(super) fn new(folder: &yazi_core::folder::Folder, window: Range<usize>) -> Self {
		Self { folder, window }
	}

	pub(super) fn register(lua: &Lua) -> mlua::Result<()> {
		lua.register_userdata_type::<Self>(|reg| {
			reg.add_meta_method(MetaMethod::Len, |_, me, ()| Ok(me.window.end - me.window.start));

			reg.add_meta_method(MetaMethod::Index, |lua, me, mut key: usize| {
				key += me.window.start;
				if key > me.window.end || key == 0 {
					Ok(None)
				} else {
					Some(lua.create_any_userdata(File::new(key - 1, me.folder()))).transpose()
				}
			});
		})?;

		Ok(())
	}

	#[inline]
	fn folder(&self) -> &yazi_core::folder::Folder { unsafe { &*self.folder } }
}
