use mlua::{IntoLua, Lua, Value};

use crate::{Composer, ComposerGet, ComposerSet};

pub fn compose(p_get: ComposerGet, p_set: ComposerSet) -> Composer<ComposerGet, ComposerSet> {
	fn get(lua: &Lua, key: &[u8]) -> mlua::Result<Value> {
		match key {
			b"Align" => super::Align::compose(lua)?,
			b"Bar" => super::Bar::compose(lua)?,
			b"Border" => super::Border::compose(lua)?,
			b"Clear" => super::Clear::compose(lua)?,
			b"Color" => super::Color::compose(lua)?,
			b"Constraint" => super::Constraint::compose(lua)?,
			b"Edge" => super::Edge::compose(lua)?,
			b"Fill" => super::Fill::compose(lua)?,
			b"Gauge" => super::Gauge::compose(lua)?,
			b"Layout" => super::Layout::compose(lua)?,
			b"Line" => super::Line::compose(lua)?,
			b"List" => super::List::compose(lua)?,
			b"Pad" => super::Pad::compose(lua)?,
			b"Pos" => super::Pos::compose(lua)?,
			b"Rect" => super::Rect::compose(lua)?,
			b"Row" => super::Row::compose(lua)?,
			b"Span" => super::Span::compose(lua)?,
			b"Style" => crate::Style::compose(lua)?,
			b"Table" => super::Table::compose(lua)?,
			b"Text" => super::Text::compose(lua)?,
			b"Wrap" => super::Wrap::compose(lua)?,
			_ => return Ok(Value::Nil),
		}
		.into_lua(lua)
	}

	fn set(_: &Lua, _: &[u8], value: Value) -> mlua::Result<Value> { Ok(value) }

	Composer::with_parent(get, set, p_get, p_set)
}
