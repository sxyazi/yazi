use std::{borrow::Cow, mem};

use ansi_to_tui::IntoText;
use mlua::{AnyUserData, ExternalError, ExternalResult, IntoLua, Lua, MetaMethod, Table, UserData, UserDataMethods, Value};
use ratatui::widgets::Widget;
use unicode_width::UnicodeWidthChar;

use super::{Area, Span};
use crate::elements::Align;

const EXPECTED: &str = "expected a string, Span, Line, or a table of them";

#[derive(Clone, Debug, Default)]
pub struct Line {
	area: Area,

	pub(super) inner: ratatui::text::Line<'static>,
}

impl Line {
	pub fn compose(lua: &Lua) -> mlua::Result<Value> {
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

		let line = lua.create_table_from([("parse", parse)])?;
		line.set_metatable(Some(lua.create_table_from([(MetaMethod::Call.name(), new)])?));
		line.into_lua(lua)
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
		yazi_binding::impl_style_shorthands!(methods, inner.style);

		methods.add_method("width", |_, me, ()| Ok(me.inner.width()));
		methods.add_function_mut("align", |_, (ud, align): (AnyUserData, Align)| {
			ud.borrow_mut::<Self>()?.inner.alignment = Some(align.0);
			Ok(ud)
		});
		methods.add_method("visible", |_, me, ()| {
			Ok(me.inner.iter().flat_map(|s| s.content.chars()).any(|c| c.width().unwrap_or(0) > 0))
		});
		methods.add_function_mut("truncate", |_, (ud, t): (AnyUserData, Table)| {
			let mut me = ud.borrow_mut::<Self>()?;

			let max = t.raw_get("max")?;
			if max < 1 {
				me.inner.spans.clear();
				return Ok(ud);
			}

			let ellipsis = match t.raw_get::<Value>("ellipsis")? {
				Value::Nil => (1, ratatui::text::Span::raw("…")),
				v => {
					let mut span = Span::try_from(v)?;
					(span.truncate(max), span.0)
				}
			};

			fn traverse(
				max: usize,
				threshold: usize,
				it: impl Iterator<Item = (usize, usize, char)>,
			) -> (Option<(usize, usize, usize)>, bool) {
				let (mut adv, mut cut) = (0, None);
				for (x, y, c) in it {
					adv += c.width().unwrap_or(0);
					if adv <= threshold {
						cut = Some((x, y, adv));
					} else if adv > max {
						break;
					}
				}
				(cut, adv > max)
			}

			let rtl = t.raw_get("rtl")?;
			let (cut, remain) = if rtl {
				traverse(
					max,
					max - ellipsis.0,
					me.inner
						.iter()
						.enumerate()
						.rev()
						.flat_map(|(x, s)| s.content.char_indices().rev().map(move |(y, c)| (x, y, c))),
				)
			} else {
				traverse(
					max,
					max - ellipsis.0,
					me.inner
						.iter()
						.enumerate()
						.flat_map(|(x, s)| s.content.char_indices().map(move |(y, c)| (x, y, c))),
				)
			};

			let Some((x, y, width)) = cut else {
				me.inner.spans.clear();
				me.inner.spans.push(ellipsis.1);
				return Ok(ud);
			};
			if !remain {
				return Ok(ud);
			}

			let spans = &mut me.inner.spans;
			let len = match (rtl, width == max) {
				(a, b) if a == b => spans[x].content[y..].chars().next().map_or(0, |c| c.len_utf8()),
				_ => 0,
			};

			if rtl {
				match &mut spans[x].content {
					Cow::Borrowed(s) => spans[x].content = Cow::Borrowed(&s[y + len..]),
					Cow::Owned(s) => _ = s.drain(..y + len),
				}
				spans.splice(..x, [ellipsis.1]);
			} else {
				match &mut spans[x].content {
					Cow::Borrowed(s) => spans[x].content = Cow::Borrowed(&s[..y + len]),
					Cow::Owned(s) => s.truncate(y + len),
				}
				spans.truncate(x + 1);
				spans.push(ellipsis.1);
			}

			Ok(ud)
		});
	}
}

#[cfg(test)]
mod tests {
	use mlua::{UserDataRef, chunk};

	use super::*;

	fn truncate(s: &str, max: usize, rtl: bool) -> String {
		let lua = Lua::new();
		let comp = Line::compose(&lua).unwrap();
		let line: UserDataRef<Line> = lua
			.load(chunk! {
				return $comp($s):truncate { max = $max, rtl = $rtl }
			})
			.call(())
			.unwrap();

		line.inner.spans.iter().map(|s| s.content.as_ref()).collect()
	}

	#[test]
	fn test_truncate() {
		assert_eq!(truncate("你好，world", 0, false), "");
		assert_eq!(truncate("你好，world", 1, false), "…");
		assert_eq!(truncate("你好，world", 2, false), "…");

		assert_eq!(truncate("你好，世界", 3, false), "你…");
		assert_eq!(truncate("你好，世界", 4, false), "你…");
		assert_eq!(truncate("你好，世界", 5, false), "你好…");

		assert_eq!(truncate("Hello, world", 5, false), "Hell…");
		assert_eq!(truncate("Ni好，世界", 3, false), "Ni…");
	}

	#[test]
	fn test_truncate_rtl() {
		assert_eq!(truncate("world，你好", 0, true), "");
		assert_eq!(truncate("world，你好", 1, true), "…");
		assert_eq!(truncate("world，你好", 2, true), "…");

		assert_eq!(truncate("你好，世界", 3, true), "…界");
		assert_eq!(truncate("你好，世界", 4, true), "…界");
		assert_eq!(truncate("你好，世界", 5, true), "…世界");

		assert_eq!(truncate("Hello, world", 5, true), "…orld");
		assert_eq!(truncate("你好，Shi界", 3, true), "…界");
	}

	#[test]
	fn test_truncate_oboe() {
		assert_eq!(truncate("Hello, world", 11, false), "Hello, wor…");
		assert_eq!(truncate("你好，世界", 9, false), "你好，世…");
		assert_eq!(truncate("你好，世Jie", 9, false), "你好，世…");

		assert_eq!(truncate("Hello, world", 11, true), "…llo, world");
		assert_eq!(truncate("你好，世界", 9, true), "…好，世界");
		assert_eq!(truncate("Ni好，世界", 9, true), "…好，世界");
	}

	#[test]
	fn test_truncate_exact() {
		assert_eq!(truncate("Hello, world", 12, false), "Hello, world");
		assert_eq!(truncate("你好，世界", 10, false), "你好，世界");

		assert_eq!(truncate("Hello, world", 12, true), "Hello, world");
		assert_eq!(truncate("你好，世界", 10, true), "你好，世界");
	}

	#[test]
	fn test_truncate_overflow() {
		assert_eq!(truncate("Hello, world", 13, false), "Hello, world");
		assert_eq!(truncate("你好，世界", 11, false), "你好，世界");

		assert_eq!(truncate("Hello, world", 13, true), "Hello, world");
		assert_eq!(truncate("你好，世界", 11, true), "你好，世界");
	}
}
