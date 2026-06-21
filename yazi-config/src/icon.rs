use mlua::{IntoLua, Lua, Value};
use serde::Deserialize;
use yazi_binding::style::{Style, StyleFlat};

#[derive(Clone, Debug, Deserialize)]
pub struct Icon {
	pub text:  String,
	#[serde(flatten)]
	pub style: StyleFlat,
}

impl IntoLua for Icon {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		lua
			.create_table_from([
				("text", self.text.into_lua(lua)?),
				("style", Style::from(self.style).into_lua(lua)?),
			])?
			.into_lua(lua)
	}
}
