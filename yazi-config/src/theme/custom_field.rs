use mlua::{IntoLua, Lua, Value};
use serde::Deserialize;
use yazi_binding::style::{Style, StyleFlat};

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum CustomField {
	Style(StyleFlat),
	String(String),
}

impl IntoLua for CustomField {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		match self {
			Self::Style(style) => Style::from(style).into_lua(lua),
			Self::String(s) => s.into_lua(lua),
		}
	}
}
