use mlua::{FromLua, Lua, Value};
use ratatui_core::{buffer::Buffer, layout::Rect, widgets::Widget};

use super::Text;

#[derive(Clone, Debug)]
pub struct Cell {
	pub(super) text: Text,
}

impl From<Cell> for ratatui_widgets::table::Cell<'static> {
	fn from(value: Cell) -> Self { Self::new(value.text.inner) }
}

impl Cell {
	pub(super) fn height(&self, width: u16) -> u16 {
		self.text.line_count(width).min(u16::MAX as usize) as u16
	}

	pub(super) fn render_overlay(&self, area: Rect, buf: &mut Buffer) {
		if self.text.needs_upgrade() {
			(&self.text).render(area, buf);
		}
	}
}

impl FromLua for Cell {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
		Ok(Self { text: Text::from_lua(value, lua)? })
	}
}
