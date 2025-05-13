use std::{borrow::Cow, mem};

use ansi_to_tui::IntoText;
use mlua::{AnyUserData, ExternalError, ExternalResult, IntoLua, Lua, MetaMethod, Table, UserData, UserDataMethods, Value};
use ratatui::widgets::Widget;
use unicode_width::UnicodeWidthChar;

use super::{Area, Span};

const LEFT: u8 = 0;
const CENTER: u8 = 1;
const RIGHT: u8 = 2;

const EXPECTED: &str = "expected a string, Span, Line, or a table of them";

#[derive(Clone, Debug, Default)]
pub struct Line {
	area: Area,

	pub(super) inner: ratatui::text::Line<'static>,
}

impl Line {
	pub fn compose(lua: &Lua) -> mlua::Result<Table> {
		let new = lua.create_function(|_, (_, value): (Table, Value)| Line::try_from(value))?;

		let parse = lua.create_function(|_, code: mlua::String| {
			let code = code.as_bytes();
			let Some(line) = code.split_inclusive(|&b| b == b'\n').next() else {
				return Ok(Line::default());
			};

			let mut lines = line.into_text().into_lua_err()?.lines;
			if lines.is_empty() {
				return Ok(Line::default());
			}

			Ok(Line { inner: mem::take(&mut lines[0]), ..Default::default() })
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

	pub(super) fn render(
		self,
		buf: &mut ratatui::buffer::Buffer,
		trans: impl Fn(yazi_config::popup::Position) -> ratatui::layout::Rect,
	) {
		let rect = self.area.transform(trans);
		self.inner.render(rect, buf);
	}
}

impl TryFrom<Value> for Line {
	type Error = mlua::Error;

	fn try_from(value: Value) -> Result<Self, Self::Error> {
		Ok(Self {
			inner: match value {
				Value::Table(tb) => return Self::try_from(tb),
				Value::String(s) => s.to_string_lossy().into(),
				Value::UserData(ud) => {
					if let Ok(Span(span)) = ud.take() {
						span.into()
					} else if let Ok(line) = ud.take() {
						return Ok(line);
					} else {
						Err(EXPECTED.into_lua_err())?
					}
				}
				_ => Err(EXPECTED.into_lua_err())?,
			},
			..Default::default()
		})
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
					} else if let Ok(Line { inner: mut line, .. }) = ud.take() {
						line.spans.iter_mut().for_each(|s| s.style = line.style.patch(s.style));
						spans.extend(line.spans);
					} else {
						return Err(EXPECTED.into_lua_err());
					}
				}
				_ => Err(EXPECTED.into_lua_err())?,
			}
		}
		Ok(Self { inner: spans.into(), ..Default::default() })
	}
}

impl From<Line> for ratatui::text::Line<'static> {
	fn from(value: Line) -> Self { value.inner }
}

impl UserData for Line {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		crate::impl_area_method!(methods);
		crate::impl_style_method!(methods, inner.style);
		crate::impl_style_shorthands!(methods, inner.style);

		methods.add_method("width", |_, me, ()| Ok(me.inner.width()));
		methods.add_function_mut("align", |_, (ud, align): (AnyUserData, u8)| {
			ud.borrow_mut::<Self>()?.inner.alignment = Some(match align {
				CENTER => ratatui::layout::Alignment::Center,
				RIGHT => ratatui::layout::Alignment::Right,
				_ => ratatui::layout::Alignment::Left,
			});
			Ok(ud)
		});
		methods.add_method("visible", |_, me, ()| {
			Ok(me.inner.iter().flat_map(|s| s.content.chars()).any(|c| c.width().unwrap_or(0) > 0))
		});
		methods.add_function_mut("truncate", |_, (ud, t): (AnyUserData, Table)| {
			let mut me = ud.borrow_mut::<Self>()?;
			let max = t.raw_get("max")?;

			let mut width = 0;
			'outer: for (x, span) in me.inner.iter_mut().enumerate() {
				for (y, c) in span.content.char_indices() {
					width += c.width().unwrap_or(0);
					if width < max {
						continue;
					} else if width == max && span.content[y..].chars().nth(1).is_none() {
						continue;
					}

					match &mut span.content {
						Cow::Borrowed(s) => span.content = Cow::Borrowed(&s[..y]),
						Cow::Owned(s) => s.truncate(y),
					}
					me.inner.spans.truncate(x + 1);
					me.inner.spans.push(ratatui::text::Span::raw("…"));
					break 'outer;
				}
			}
			Ok(ud)
		});
	}
}
