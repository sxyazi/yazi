use mlua::{ExternalError, FromLua};

use super::Text;

const EXPECTED: &str = "expected a table of strings, Texts, Lines or Spans";

#[derive(Clone, Debug)]
pub struct Cell {
	pub(super) text: ratatui::text::Text<'static>,
}

impl From<Cell> for ratatui::widgets::Cell<'static> {
	fn from(value: Cell) -> Self { Self::new(value.text) }
}

impl FromLua for Cell {
	fn from_lua(value: mlua::Value, _: &mlua::Lua) -> mlua::Result<Self> {
		Ok(Self { text: Text::try_from(value).map_err(|_| EXPECTED.into_lua_err())?.inner })
	}
}
