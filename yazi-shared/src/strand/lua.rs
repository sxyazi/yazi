use mlua::{IntoLua, Lua, Value};

use crate::strand::{StrandBuf, StrandLike};

impl IntoLua for StrandBuf {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		lua.create_external_string(self.into_encoded_bytes())?.into_lua(lua)
	}
}

impl IntoLua for &StrandBuf {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		lua.create_string(self.encoded_bytes())?.into_lua(lua)
	}
}
