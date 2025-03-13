use mlua::{AnyUserData, ExternalError, FromLua, IntoLua, Lua, Table, UserDataRef, Value};

use crate::{bindings::Cha, impl_file_fields, impl_file_methods};

pub type FileRef = UserDataRef<yazi_fs::File>;

pub struct File(pub yazi_fs::File);

impl File {
	#[inline]
	pub fn register(lua: &Lua) -> mlua::Result<()> {
		lua.register_userdata_type::<yazi_fs::File>(|reg| {
			impl_file_fields!(reg);
			impl_file_methods!(reg);
		})
	}

	pub fn install(lua: &Lua) -> mlua::Result<()> {
		lua.globals().raw_set(
			"File",
			lua.create_function(|_, t: Table| {
				Ok(Self(yazi_fs::File {
					url: t.raw_get::<AnyUserData>("url")?.take()?,
					cha: *t.raw_get::<Cha>("cha")?,
					..Default::default()
				}))
			})?,
		)
	}
}

impl FromLua for File {
	fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
		match value {
			Value::UserData(ud) => ud.take().map(Self),
			_ => Err("Expected a File".into_lua_err()),
		}
	}
}

impl IntoLua for File {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		lua.create_any_userdata(self.0)?.into_lua(lua)
	}
}
