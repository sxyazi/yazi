use mlua::{AnyUserData, IntoLua, Lua, Value};
use tracing::error;

use super::Renderable;
use crate::Composer;

pub fn compose(lua: &Lua) -> mlua::Result<Value> {
	Composer::make(lua, 20, |lua, key| {
		match key {
			b"Align" => super::Align::compose(lua)?,
			b"Bar" => super::Bar::compose(lua)?,
			b"Border" => super::Border::compose(lua)?,
			b"Clear" => super::Clear::compose(lua)?,
			b"Constraint" => super::Constraint::compose(lua)?,
			b"Edge" => super::Edge::compose(lua)?,
			b"Gauge" => super::Gauge::compose(lua)?,
			b"Layout" => super::Layout::compose(lua)?,
			b"Line" => super::Line::compose(lua)?,
			b"List" => super::List::compose(lua)?,
			b"Pad" => super::Pad::compose(lua)?,
			b"Pos" => super::Pos::compose(lua)?,
			b"Rect" => super::Rect::compose(lua)?,
			b"Row" => super::Row::compose(lua)?,
			b"Span" => super::Span::compose(lua)?,
			b"Style" => super::Style::compose(lua)?,
			b"Table" => super::Table::compose(lua)?,
			b"Text" => super::Text::compose(lua)?,
			b"Wrap" => super::Wrap::compose(lua)?,

			b"width" => super::Utils::width(lua)?,
			b"redraw" => super::Utils::redraw(lua)?,
			_ => return Ok(Value::Nil),
		}
		.into_lua(lua)
	})
}

pub fn render_once<F>(value: Value, buf: &mut ratatui::buffer::Buffer, trans: F)
where
	F: Fn(yazi_config::popup::Position) -> ratatui::layout::Rect + Copy,
{
	match value {
		Value::Table(tbl) => {
			for widget in tbl.sequence_values::<AnyUserData>() {
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
		Value::UserData(ud) => match Renderable::try_from(ud) {
			Ok(w) => w.render(buf, trans),
			Err(e) => error!("{e}"),
		},
		_ => error!("Expected a renderable UserData, or a table of them, got: {value:?}"),
	}
}
