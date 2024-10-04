use mlua::{FromLua, Lua, Table, UserData};

#[derive(Clone, Copy, FromLua)]
pub struct Constraint(pub(super) ratatui::layout::Constraint);

impl Constraint {
	pub fn install(lua: &Lua, ui: &Table) -> mlua::Result<()> {
		let constraint = lua.create_table_from([
			(
				"Min",
				lua.create_function(|_, n: u16| Ok(Constraint(ratatui::layout::Constraint::Min(n))))?,
			),
			(
				"Max",
				lua.create_function(|_, n: u16| Ok(Constraint(ratatui::layout::Constraint::Max(n))))?,
			),
			(
				"Length",
				lua.create_function(|_, n: u16| Ok(Constraint(ratatui::layout::Constraint::Length(n))))?,
			),
			(
				"Percentage",
				lua.create_function(|_, n: u16| {
					Ok(Constraint(ratatui::layout::Constraint::Percentage(n)))
				})?,
			),
			(
				"Ratio",
				lua.create_function(|_, (a, b): (u32, u32)| {
					Ok(Constraint(ratatui::layout::Constraint::Ratio(a, b)))
				})?,
			),
			(
				"Fill",
				lua.create_function(|_, n: u16| Ok(Constraint(ratatui::layout::Constraint::Fill(n))))?,
			),
		])?;

		ui.raw_set("Constraint", constraint)
	}
}

impl UserData for Constraint {}
