use mlua::{FromLua, Lua, Value};

use super::Text;

#[derive(Clone, Debug)]
pub struct Cell {
	pub(super) text: ratatui::text::Text<'static>,
}

impl From<Cell> for ratatui::widgets::Cell<'static> {
	fn from(value: Cell) -> Self { Self::new(value.text) }
}

impl FromLua for Cell {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
		Ok(Self { text: Text::from_lua(value, lua)?.inner })
	}
}
