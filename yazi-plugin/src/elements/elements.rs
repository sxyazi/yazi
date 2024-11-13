use mlua::{AnyUserData, IntoLua, Lua, Table, Value};
use tracing::error;

use crate::cast_to_renderable;

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
			b"Padding" => super::Padding::compose(lua)?,
			b"Paragraph" => super::Paragraph::compose(lua)?,
			b"Position" => super::Position::compose(lua)?,
			b"Rect" => super::Rect::compose(lua)?,
			b"Span" => super::Span::compose(lua)?,
			b"Style" => super::Style::compose(lua)?,
			b"Table" => super::Table::compose(lua)?,
			b"TableRow" => super::TableRow::compose(lua)?,
			b"Text" => super::Text::compose(lua)?,
			_ => return Ok(Value::Nil),
		}
		.into_lua(lua)?;

		ts.raw_set(key, value.clone())?;
		Ok(value)
	})?;

	let ui = lua.create_table_with_capacity(0, 20)?;
	ui.set_metatable(Some(lua.create_table_from([("__index", index)])?));

	Ok(ui)
}

pub trait Renderable {
	fn area(&self) -> ratatui::layout::Rect;

	fn render(self: Box<Self>, buf: &mut ratatui::buffer::Buffer);

	fn clone_render(&self, buf: &mut ratatui::buffer::Buffer);
}

pub fn render_widgets(widgets: Table, buf: &mut ratatui::buffer::Buffer) {
	for widget in widgets.sequence_values::<AnyUserData>() {
		let Ok(widget) = widget else {
			error!("Failed to convert to renderable UserData: {}", widget.unwrap_err());
			continue;
		};

		match cast_to_renderable(&widget) {
			Some(w) => w.render(buf),
			None => error!("Only the UserData of renderable element is accepted: {widget:#?}"),
		}
	}
}
