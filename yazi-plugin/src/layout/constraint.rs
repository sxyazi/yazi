use mlua::{FromLua, Lua, Table, UserData, Value};

use crate::{GLOBALS, LUA};

#[derive(Clone, Copy)]
pub(crate) struct Constraint(pub(super) ratatui::layout::Constraint);

impl Constraint {
	pub(crate) fn install() -> mlua::Result<()> {
		let ui: Table = GLOBALS.get("ui")?;

		let constraint = LUA.create_table()?;
		constraint.set(
			"Percentage",
			LUA
				.create_function(|_, n: u16| Ok(Constraint(ratatui::layout::Constraint::Percentage(n))))?,
		)?;
		constraint.set(
			"Ratio",
			LUA.create_function(|_, (a, b): (u32, u32)| {
				Ok(Constraint(ratatui::layout::Constraint::Ratio(a, b)))
			})?,
		)?;
		constraint.set(
			"Length",
			LUA.create_function(|_, n: u16| Ok(Constraint(ratatui::layout::Constraint::Length(n))))?,
		)?;
		constraint.set(
			"Max",
			LUA.create_function(|_, n: u16| Ok(Constraint(ratatui::layout::Constraint::Max(n))))?,
		)?;
		constraint.set(
			"Min",
			LUA.create_function(|_, n: u16| Ok(Constraint(ratatui::layout::Constraint::Min(n))))?,
		)?;

		ui.set("Constraint", constraint)
	}
}

impl<'lua> FromLua<'lua> for Constraint {
	fn from_lua(value: Value<'lua>, _: &'lua Lua) -> mlua::Result<Self> {
		match value {
			Value::UserData(ud) => Ok(*ud.borrow::<Self>()?),
			_ => Err(mlua::Error::FromLuaConversionError {
				from:    value.type_name(),
				to:      "Constraint",
				message: Some("expected a Constraint".to_string()),
			}),
		}
	}
}

impl UserData for Constraint {}
