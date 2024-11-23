use mlua::{AnyUserData, IntoLua, Lua, MetaMethod, Table, Value};
use tracing::error;

use super::Renderable;

pub fn compose(lua: &Lua) -> mlua::Result<Table> {
	let index = lua.create_function(|lua, (ts, key): (Table, mlua::String)| {
		let value = match key.as_bytes().as_ref() {
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
			b"Paragraph" => super::Paragraph::compose(lua)?,
			b"Pos" => super::Pos::compose(lua)?,
			b"Rect" => super::Rect::compose(lua)?,
			b"Row" => super::Row::compose(lua)?,
			b"Span" => super::Span::compose(lua)?,
			b"Style" => super::Style::compose(lua)?,
			b"Table" => super::Table::compose(lua)?,
			b"Text" => super::Text::compose(lua)?,
			_ => return Ok(Value::Nil),
		}
		.into_lua(lua)?;

		ts.raw_set(key, value.clone())?;
		Ok(value)
	})?;

	let ui = lua.create_table_with_capacity(0, 20)?;
	ui.set_metatable(Some(lua.create_table_from([(MetaMethod::Index.name(), index)])?));

	Ok(ui)
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
