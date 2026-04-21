use std::ops::Deref;

use mlua::{IntoLua, Lua, Value};

use crate::Style;

pub struct Icon {
	inner: yazi_config::Icon,
}

impl Deref for Icon {
	type Target = yazi_config::Icon;

	fn deref(&self) -> &Self::Target { &self.inner }
}

impl From<yazi_config::Icon> for Icon {
	fn from(inner: yazi_config::Icon) -> Self { Self { inner } }
}

impl IntoLua for Icon {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		lua
			.create_table_from([
				("text", self.inner.text.into_lua(lua)?),
				("style", Style::from(self.inner.style).into_lua(lua)?),
			])?
			.into_lua(lua)
	}
}
