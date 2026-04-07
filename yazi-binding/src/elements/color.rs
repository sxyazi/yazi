use std::str::FromStr;

use mlua::{ExternalError, ExternalResult, FromLua, IntoLua, Lua, MetaMethod, Table, UserData, Value};

#[derive(Clone, Copy, Default)]
pub struct Color(pub ratatui::style::Color);

impl Color {
	pub fn compose(lua: &Lua) -> mlua::Result<Value> {
		let new = lua.create_function(|_, (_, color): (Table, Color)| Ok(color))?;

		let color = lua.create_table()?;
		color.set_metatable(Some(lua.create_table_from([(MetaMethod::Call.name(), new)])?))?;

		color.into_lua(lua)
	}
}

impl FromLua for Color {
	fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
		Ok(Self(match value {
			Value::Nil => ratatui::style::Color::Reset,
			Value::String(s) => ratatui::style::Color::from_str(&s.to_str()?).into_lua_err()?,
			Value::UserData(ud) => ud.borrow::<Self>()?.0,
			_ => Err("expected a Color".into_lua_err())?,
		}))
	}
}

impl UserData for Color {}
