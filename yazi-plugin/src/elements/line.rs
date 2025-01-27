use std::mem;

use ansi_to_tui::IntoText;
use mlua::{AnyUserData, ExternalError, ExternalResult, IntoLua, Lua, MetaMethod, Table, UserData, UserDataMethods, Value};
use unicode_width::UnicodeWidthChar;

use super::Span;

const LEFT: u8 = 0;
const CENTER: u8 = 1;
const RIGHT: u8 = 2;

const EXPECTED: &str = "expected a string, Span, Line, or a table of them";

pub struct Line(pub(super) ratatui::text::Line<'static>);

impl Line {
	pub fn compose(lua: &Lua) -> mlua::Result<Table> {
		let new = lua.create_function(|_, (_, value): (Table, Value)| Line::try_from(value))?;

		let parse = lua.create_function(|_, code: mlua::String| {
			let code = code.as_bytes();
			let Some(line) = code.split_inclusive(|&b| b == b'\n').next() else {
				return Ok(Line(Default::default()));
			};

			let mut lines = line.into_text().into_lua_err()?.lines;
			if lines.is_empty() {
				return Ok(Line(Default::default()));
			}

			Ok(Line(mem::take(&mut lines[0])))
		})?;

		let line = lua.create_table_from([
			("parse", parse.into_lua(lua)?),
			// Alignment
			("LEFT", LEFT.into_lua(lua)?),
			("CENTER", CENTER.into_lua(lua)?),
			("RIGHT", RIGHT.into_lua(lua)?),
		])?;

		line.set_metatable(Some(lua.create_table_from([(MetaMethod::Call.name(), new)])?));
		Ok(line)
	}
}

impl TryFrom<Value> for Line {
	type Error = mlua::Error;

	fn try_from(value: Value) -> Result<Self, Self::Error> {
		Ok(Self(match value {
			Value::Table(tb) => return Self::try_from(tb),
			Value::String(s) => s.to_string_lossy().into(),
			Value::UserData(ud) => {
				if let Ok(Span(span)) = ud.take() {
					span.into()
				} else if let Ok(Line(mut line)) = ud.take() {
					line.spans.iter_mut().for_each(|s| s.style = line.style.patch(s.style));
					line
				} else {
					Err(EXPECTED.into_lua_err())?
				}
			}
			_ => Err(EXPECTED.into_lua_err())?,
		}))
	}
}

impl TryFrom<Table> for Line {
	type Error = mlua::Error;

	fn try_from(tb: Table) -> Result<Self, Self::Error> {
		let mut spans = Vec::with_capacity(tb.raw_len());
		for v in tb.sequence_values() {
			match v? {
				Value::String(s) => spans.push(s.to_string_lossy().into()),
				Value::UserData(ud) => {
					if let Ok(Span(span)) = ud.take() {
						spans.push(span);
					} else if let Ok(Line(mut line)) = ud.take() {
						line.spans.iter_mut().for_each(|s| s.style = line.style.patch(s.style));
						spans.extend(line.spans);
					} else {
						return Err(EXPECTED.into_lua_err());
					}
				}
				_ => Err(EXPECTED.into_lua_err())?,
			}
		}
		Ok(Self(spans.into()))
	}
}

impl From<Line> for ratatui::text::Line<'static> {
	fn from(value: Line) -> Self { value.0 }
}

impl UserData for Line {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		crate::impl_style_method!(methods, 0.style);
		crate::impl_style_shorthands!(methods, 0.style);

		methods.add_method("width", |_, Line(me), ()| Ok(me.width()));
		methods.add_function_mut("align", |_, (ud, align): (AnyUserData, u8)| {
			ud.borrow_mut::<Self>()?.0.alignment = Some(match align {
				CENTER => ratatui::layout::Alignment::Center,
				RIGHT => ratatui::layout::Alignment::Right,
				_ => ratatui::layout::Alignment::Left,
			});
			Ok(ud)
		});
		methods.add_method("visible", |_, Line(me), ()| {
			Ok(me.iter().flat_map(|s| s.content.chars()).any(|c| c.width().unwrap_or(0) > 0))
		});
	}
}
