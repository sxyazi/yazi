use mlua::{IntoLua, Lua, Value};

pub struct Source(pub yazi_shared::Source);

impl IntoLua for Source {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> { self.0.bits().into_lua(lua) }
}
