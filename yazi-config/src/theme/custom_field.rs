use mlua::{IntoLua, Lua, Value};
use serde::{Deserialize, Deserializer, de};
use yazi_binding::{elements::BorderSet, position::Position, style::Style};

#[derive(Clone, Debug)]
pub enum CustomField {
	BorderSet(BorderSet),
	Position(Position),
	Style(Style),
	String(String),
}

impl<'de> Deserialize<'de> for CustomField {
	fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
		let value = toml::Value::deserialize(de)?;

		match value {
			toml::Value::Table(ref t) if t.contains_key("as") => {
				BorderSet::deserialize(value).map(Self::BorderSet)
			}
			toml::Value::Table(ref t) if t.contains_key("at") => {
				Position::deserialize(value).map(Self::Position)
			}
			toml::Value::String(s) => Ok(Self::String(s)),
			v => Style::deserialize(v).map(Self::Style),
		}
		.map_err(de::Error::custom)
	}
}

impl IntoLua for CustomField {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		match self {
			Self::BorderSet(s) => s.into_lua(lua),
			Self::Position(p) => p.into_lua(lua),
			Self::Style(s) => s.into_lua(lua),
			Self::String(s) => s.into_lua(lua),
		}
	}
}
