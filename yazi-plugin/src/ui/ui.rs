use mlua::{IntoLua, Lua, Value};
use yazi_binding::{Composer, ComposerGet, ComposerSet, elements::{Align, Bar, Border, Color, Constraint, Edge, Fill, Gauge, Layout, Line, List, Pad, Rect, Row, Span, Table, Text, Wrap}, position::Position, style::Style};
use yazi_config::THEME;
use yazi_widgets::{clear::Clear, input::InputArc};

pub fn compose() -> Composer<ComposerGet, ComposerSet> {
	fn get(lua: &Lua, key: &[u8]) -> mlua::Result<Value> {
		match key {
			// Elements
			b"Align" => Align::compose(lua)?,
			b"Bar" => Bar::compose(lua)?,
			b"Border" => Border::compose(lua)?,
			b"Clear" => Clear::compose(lua)?,
			b"Color" => Color::compose(lua)?,
			b"Constraint" => Constraint::compose(lua)?,
			b"Edge" => Edge::compose(lua)?,
			b"Fill" => Fill::compose(lua)?,
			b"Gauge" => Gauge::compose(lua)?,
			b"Input" => InputArc::compose(lua, (&THEME.input).into())?,
			b"Layout" => Layout::compose(lua)?,
			b"Line" => Line::compose(lua)?,
			b"List" => List::compose(lua)?,
			b"Pad" => Pad::compose(lua)?,
			b"Pos" => Position::compose(lua)?,
			b"Rect" => Rect::compose(lua)?,
			b"Row" => Row::compose(lua)?,
			b"Span" => Span::compose(lua)?,
			b"Style" => Style::compose(lua)?,
			b"Table" => Table::compose(lua)?,
			b"Text" => Text::compose(lua)?,
			b"Wrap" => Wrap::compose(lua)?,

			// Functions
			b"area" => super::area(lua)?,
			b"hide" => super::hide(lua)?,
			b"lines" => super::lines(lua)?,
			b"printable" => super::printable(lua)?,
			b"redraw" => super::redraw(lua)?,
			b"render" => super::render(lua)?,
			b"truncate" => super::truncate(lua)?,
			b"width" => super::width(lua)?,
			_ => return Ok(Value::Nil),
		}
		.into_lua(lua)
	}

	fn set(_: &Lua, _: &[u8], value: Value) -> mlua::Result<Value> { Ok(value) }

	Composer::new(get, set)
}
