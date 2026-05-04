use mlua::IntoLua;

use crate::Style;

pub struct CustomField(yazi_config::theme::CustomField);

impl CustomField {
	pub fn new(inner: impl Into<yazi_config::theme::CustomField>) -> Self { Self(inner.into()) }
}

impl IntoLua for CustomField {
	fn into_lua(self, lua: &mlua::Lua) -> mlua::Result<mlua::Value> {
		match self.0 {
			yazi_config::theme::CustomField::Style(style) => Style::from(style).into_lua(lua),
			yazi_config::theme::CustomField::String(s) => s.into_lua(lua),
		}
	}
}
