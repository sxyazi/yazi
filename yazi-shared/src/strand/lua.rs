use mlua::{IntoLua, Lua, Value};

use crate::strand::StrandBuf;

impl IntoLua for StrandBuf {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		lua.create_external_string(self.into_encoded_bytes())?.into_lua(lua)
	}
}
