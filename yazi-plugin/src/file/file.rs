use mlua::{AnyUserData, Lua, Table, UserDataRef};

use crate::{bindings::Cast, impl_file_fields, impl_file_methods};

pub type FileRef = UserDataRef<yazi_shared::fs::File>;

pub struct File;

impl File {
	#[inline]
	pub fn register(lua: &Lua) -> mlua::Result<()> {
		lua.register_userdata_type::<yazi_shared::fs::File>(|reg| {
			impl_file_fields!(reg);
			impl_file_methods!(reg);
		})
	}

	pub fn install(lua: &Lua) -> mlua::Result<()> {
		lua.globals().raw_set(
			"File",
			lua.create_function(|lua, t: Table| {
				Self::cast(lua, yazi_shared::fs::File {
					url: t.raw_get::<AnyUserData>("url")?.take()?,
					cha: t.raw_get::<AnyUserData>("cha")?.take()?,
					..Default::default()
				})
			})?,
		)
	}
}

impl<T: Into<yazi_shared::fs::File>> Cast<T> for File {
	fn cast(lua: &Lua, data: T) -> mlua::Result<AnyUserData> { lua.create_any_userdata(data.into()) }
}
