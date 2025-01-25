use mlua::{AnyUserData, IntoLua, Lua, Table, Value};
use tracing::error;

use super::Renderable;
use crate::Composer;

pub fn compose(lua: &Lua) -> mlua::Result<Value> {
	Composer::make(lua, 20, |lua, key| {
		match key {
			b"Bar" => super::Bar::compose(lua)?,
			b"Border" => super::Border::compose(lua)?,
			b"Clear" => super::Clear::compose(lua)?,
			b"Constraint" => super::Constraint::compose(lua)?,
			b"Gauge" => super::Gauge::compose(lua)?,
			b"Layout" => super::Layout::compose(lua)?,
			b"Line" => super::Line::compose(lua)?,
			b"List" => super::List::compose(lua)?,
			b"Pad" => super::Pad::compose(lua, true)?,
			// TODO: deprecate this
			b"Padding" => super::Pad::compose(lua, false)?,
			b"Pos" => super::Pos::compose(lua)?,
			b"Rect" => super::Rect::compose(lua)?,
			b"Row" => super::Row::compose(lua)?,
			b"Span" => super::Span::compose(lua)?,
			b"Style" => super::Style::compose(lua)?,
			b"Table" => super::Table::compose(lua)?,
			b"Text" => super::Text::compose(lua)?,
			_ => return Ok(Value::Nil),
		}
		.into_lua(lua)
	})
}

pub fn render_once<F>(widgets: Table, buf: &mut ratatui::buffer::Buffer, trans: F)
where
	F: Fn(yazi_config::popup::Position) -> ratatui::layout::Rect + Copy,
{
	for widget in widgets.sequence_values::<AnyUserData>() {
		let Ok(widget) = widget else {
			error!("Failed to convert to renderable UserData: {}", widget.unwrap_err());
			continue;
		};

		match Renderable::try_from(widget) {
			Ok(w) => w.render(buf, trans),
			Err(e) => error!("{e}"),
		}
	}
}
