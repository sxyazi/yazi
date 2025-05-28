use std::ops::Deref;

use mlua::{FromLua, IntoLua, Lua, Value};
use yazi_config::preview::PreviewWrap;

#[derive(Clone, Copy, Debug, Default)]
pub struct Wrap(pub(super) Option<ratatui::widgets::Wrap>);

impl Deref for Wrap {
	type Target = Option<ratatui::widgets::Wrap>;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl Wrap {
	pub fn compose(lua: &Lua) -> mlua::Result<Value> {
		lua.create_table_from([("NO", 0), ("YES", 1), ("TRIM", 2)])?.into_lua(lua)
	}
}

impl From<PreviewWrap> for Wrap {
	fn from(value: PreviewWrap) -> Self {
		Self(match value {
			PreviewWrap::No => None,
			PreviewWrap::Yes => Some(ratatui::widgets::Wrap { trim: false }),
		})
	}
}

impl FromLua for Wrap {
	fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
		let Value::Integer(n) = value else {
			return Err(mlua::Error::FromLuaConversionError {
				from:    value.type_name(),
				to:      "Wrap".to_string(),
				message: Some("expected an integer representation of a Wrap".to_string()),
			});
		};
		Ok(Self(match n {
			0 => None,
			1 => Some(ratatui::widgets::Wrap { trim: false }),
			2 => Some(ratatui::widgets::Wrap { trim: true }),
			_ => Err(mlua::Error::FromLuaConversionError {
				from:    value.type_name(),
				to:      "Wrap".to_string(),
				message: Some("invalid value for Wrap".to_string()),
			})?,
		}))
	}
}
