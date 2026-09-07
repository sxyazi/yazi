use mlua::{IntoLua, Lua, Value};
use serde::Deserialize;
use yazi_binding::{position::Position, style::Style};

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum CustomField {
	Pos(Position),
	Style(Style),
	String(String),
}

impl IntoLua for CustomField {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		match self {
			Self::Pos(pos) => pos.into_lua(lua),
			Self::Style(style) => style.into_lua(lua),
			Self::String(s) => s.into_lua(lua),
		}
	}
}
