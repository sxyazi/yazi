use mlua::{FromLua, Lua, Table, UserData};

#[derive(Clone, Copy, FromLua)]
pub struct Constraint(pub(super) ratatui::layout::Constraint);

impl Constraint {
	pub fn install(lua: &Lua, ui: &Table) -> mlua::Result<()> {
		let constraint = lua.create_table()?;

		constraint.raw_set(
			"Min",
			lua.create_function(|_, n: u16| Ok(Constraint(ratatui::layout::Constraint::Min(n))))?,
		)?;
		constraint.raw_set(
			"Max",
			lua.create_function(|_, n: u16| Ok(Constraint(ratatui::layout::Constraint::Max(n))))?,
		)?;
		constraint.raw_set(
			"Length",
			lua.create_function(|_, n: u16| Ok(Constraint(ratatui::layout::Constraint::Length(n))))?,
		)?;
		constraint.raw_set(
			"Percentage",
			lua
				.create_function(|_, n: u16| Ok(Constraint(ratatui::layout::Constraint::Percentage(n))))?,
		)?;
		constraint.raw_set(
			"Ratio",
			lua.create_function(|_, (a, b): (u32, u32)| {
				Ok(Constraint(ratatui::layout::Constraint::Ratio(a, b)))
			})?,
		)?;
		constraint.raw_set(
			"Fill",
			lua.create_function(|_, n: u16| Ok(Constraint(ratatui::layout::Constraint::Fill(n))))?,
		)?;

		ui.raw_set("Constraint", constraint)
	}
}

impl UserData for Constraint {}
