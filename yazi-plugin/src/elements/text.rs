use std::mem;

use ansi_to_tui::IntoText;
use mlua::{ExternalError, ExternalResult, IntoLua, Lua, MetaMethod, Table, UserData, Value};
use ratatui::widgets::Widget;

use super::{Area, Line, Span};

// Alignment
pub(super) const LEFT: u8 = 0;
pub(super) const CENTER: u8 = 1;
pub(super) const RIGHT: u8 = 2;

// Wrap
pub const WRAP_NO: u8 = 0;
pub const WRAP: u8 = 1;
pub const WRAP_TRIM: u8 = 2;

const EXPECTED: &str = "expected a string, Line, Span, or a table of them";

#[derive(Clone, Debug, Default)]
pub struct Text {
	pub area: Area,

	// TODO: block
	pub inner: ratatui::text::Text<'static>,
	pub wrap:  u8,
	// TODO: scroll
}

impl Text {
	pub fn compose(lua: &Lua) -> mlua::Result<Table> {
		let new = lua.create_function(|_, (_, value): (Table, Value)| Text::try_from(value))?;

		let parse = lua.create_function(|_, code: mlua::String| {
			Ok(Text { inner: code.as_bytes().into_text().into_lua_err()?, ..Default::default() })
		})?;

		let text = lua.create_table_from([
			("parse", parse.into_lua(lua)?),
			// Alignment
			("LEFT", LEFT.into_lua(lua)?),
			("CENTER", CENTER.into_lua(lua)?),
			("RIGHT", RIGHT.into_lua(lua)?),
			// Wrap
			("WRAP_NO", WRAP_NO.into_lua(lua)?),
			("WRAP", WRAP.into_lua(lua)?),
			("WRAP_TRIM", WRAP_TRIM.into_lua(lua)?),
		])?;

		text.set_metatable(Some(lua.create_table_from([(MetaMethod::Call.name(), new)])?));
		Ok(text)
	}

	pub(super) fn render(
		self,
		buf: &mut ratatui::buffer::Buffer,
		trans: impl Fn(yazi_config::popup::Position) -> ratatui::layout::Rect,
	) {
		let rect = self.area.transform(trans);
		if self.wrap == WRAP_NO {
			self.inner.render(rect, buf);
		} else {
			ratatui::widgets::Paragraph::from(self).render(rect, buf);
		}
	}
}

impl TryFrom<Value> for Text {
	type Error = mlua::Error;

	fn try_from(value: Value) -> mlua::Result<Self> {
		let inner = match value {
			Value::Table(tb) => return Self::try_from(tb),
			Value::String(s) => s.to_string_lossy().into(),
			Value::UserData(ud) => {
				if let Ok(Line(line)) = ud.take() {
					line.into()
				} else if let Ok(Span(span)) = ud.take() {
					span.into()
				} else if let Ok(text) = ud.take() {
					return Ok(text);
				} else {
					Err(EXPECTED.into_lua_err())?
				}
			}
			_ => Err(EXPECTED.into_lua_err())?,
		};
		Ok(Self { inner, ..Default::default() })
	}
}

impl TryFrom<Table> for Text {
	type Error = mlua::Error;

	fn try_from(tb: Table) -> Result<Self, Self::Error> {
		let mut lines = Vec::with_capacity(tb.raw_len());
		for v in tb.sequence_values() {
			match v? {
				Value::String(s) => lines.push(s.to_string_lossy().into()),
				Value::UserData(ud) => {
					if let Ok(Span(span)) = ud.take() {
						lines.push(span.into());
					} else if let Ok(Line(line)) = ud.take() {
						lines.push(line);
					} else {
						return Err(EXPECTED.into_lua_err());
					}
				}
				_ => Err(EXPECTED.into_lua_err())?,
			}
		}
		Ok(Self { inner: lines.into(), ..Default::default() })
	}
}

impl From<Text> for ratatui::text::Text<'static> {
	fn from(value: Text) -> Self { value.inner }
}

impl From<Text> for ratatui::widgets::Paragraph<'static> {
	fn from(mut value: Text) -> Self {
		let align = value.inner.alignment.take();
		let style = mem::take(&mut value.inner.style);

		let mut p = ratatui::widgets::Paragraph::new(value.inner).style(style);
		if let Some(align) = align {
			p = p.alignment(align);
		}
		if value.wrap != WRAP_NO {
			p = p.wrap(ratatui::widgets::Wrap { trim: value.wrap == WRAP_TRIM });
		}
		p
	}
}

impl UserData for Text {
	fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
		crate::impl_area_method!(methods);
		crate::impl_style_method!(methods, inner.style);
		crate::impl_style_shorthands!(methods, inner.style);

		methods.add_function_mut("align", |_, (ud, align): (AnyUserData, u8)| {
			ud.borrow_mut::<Self>()?.inner.alignment = Some(match align {
				CENTER => ratatui::layout::Alignment::Center,
				RIGHT => ratatui::layout::Alignment::Right,
				_ => ratatui::layout::Alignment::Left,
			});
			Ok(ud)
		});
		methods.add_function_mut("wrap", |_, (ud, wrap): (AnyUserData, u8)| {
			ud.borrow_mut::<Self>()?.wrap = match wrap {
				w @ (WRAP | WRAP_TRIM | WRAP_NO) => w,
				_ => return Err("expected a WRAP, WRAP_TRIM or WRAP_NO".into_lua_err()),
			};
			Ok(ud)
		});
		methods.add_method("max_width", |_, me, ()| {
			Ok(me.inner.lines.iter().take(me.area.size().height as usize).map(|l| l.width()).max())
		});
	}
}
