use mlua::{FromLua, Lua, Table, UserData};

#[derive(Clone, Copy, FromLua)]
pub struct Constraint(pub(super) ratatui::layout::Constraint);

impl Constraint {
	pub fn install(lua: &Lua, ui: &Table) -> mlua::Result<()> {
		let constraint = lua.create_table()?;

		constraint.set(
			"Percentage",
			lua
				.create_function(|_, n: u16| Ok(Constraint(ratatui::layout::Constraint::Percentage(n))))?,
		)?;
		constraint.set(
			"Ratio",
			lua.create_function(|_, (a, b): (u32, u32)| {
				Ok(Constraint(ratatui::layout::Constraint::Ratio(a, b)))
			})?,
		)?;
		constraint.set(
			"Length",
			lua.create_function(|_, n: u16| Ok(Constraint(ratatui::layout::Constraint::Length(n))))?,
		)?;
		constraint.set(
			"Max",
			lua.create_function(|_, n: u16| Ok(Constraint(ratatui::layout::Constraint::Max(n))))?,
		)?;
		constraint.set(
			"Min",
			lua.create_function(|_, n: u16| Ok(Constraint(ratatui::layout::Constraint::Min(n))))?,
		)?;

		ui.set("Constraint", constraint)
	}
}

impl UserData for Constraint {}
