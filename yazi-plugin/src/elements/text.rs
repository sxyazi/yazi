use ansi_to_tui::IntoText;
use mlua::{AnyUserData, ExternalError, ExternalResult, FromLua, IntoLua, Lua, Table, UserData, Value};
use ratatui::widgets::Widget;

use super::{Line, Rect, Renderable, Span};

// Alignment
pub(super) const LEFT: u8 = 0;
pub(super) const CENTER: u8 = 1;
pub(super) const RIGHT: u8 = 2;

// Wrap
pub const WRAP_NO: u8 = 0;
pub const WRAP: u8 = 1;
pub const WRAP_TRIM: u8 = 2;

const EXPECTED: &str = "expected a string, ui.Line, ui.Span or a table of them";

#[derive(Clone, Default, FromLua)]
pub struct Text {
	pub area: Rect,

	// TODO: block
	pub inner: ratatui::text::Text<'static>,
	pub wrap:  u8,
	// TODO: scroll
}

impl Text {
	pub fn install(lua: &Lua, ui: &Table) -> mlua::Result<()> {
		let new = lua.create_function(|_, (_, value): (Table, Value)| Text::try_from(value))?;

		let parse = lua.create_function(|_, code: mlua::String| {
			Ok(Text { inner: code.into_text().into_lua_err()?, ..Default::default() })
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

		text.set_metatable(Some(lua.create_table_from([("__call", new)])?));

		ui.raw_set("Text", text)
	}
}

impl TryFrom<Value<'_>> for Text {
	type Error = mlua::Error;

	fn try_from(value: Value) -> mlua::Result<Self> {
		let inner = match value {
			Value::Table(tb) => return Self::try_from(tb),
			Value::String(s) => s.to_string_lossy().into_owned().into(),
			Value::UserData(ud) => {
				if let Ok(line) = ud.take::<Line>() {
					line.0.into()
				} else if let Ok(span) = ud.take::<Span>() {
					span.0.into()
				} else {
					Err(EXPECTED.into_lua_err())?
				}
			}
			_ => Err(EXPECTED.into_lua_err())?,
		};
		Ok(Self { inner, ..Default::default() })
	}
}

impl TryFrom<Table<'_>> for Text {
	type Error = mlua::Error;

	fn try_from(tb: Table<'_>) -> Result<Self, Self::Error> {
		let mut lines = Vec::with_capacity(tb.raw_len());
		for v in tb.sequence_values() {
			match v? {
				Value::String(s) => lines.push(s.to_string_lossy().into_owned().into()),
				Value::UserData(ud) => {
					if let Ok(span) = ud.take::<Span>() {
						lines.push(span.0.into());
					} else if let Ok(line) = ud.take::<Line>() {
						lines.push(line.0);
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
	fn from(value: Text) -> Self {
		let align = value.inner.alignment.unwrap_or_default();
		let mut p = ratatui::widgets::Paragraph::new(value.inner);

		if value.wrap != WRAP_NO {
			p = p.wrap(ratatui::widgets::Wrap { trim: value.wrap == WRAP_TRIM });
		}

		p.alignment(align)
	}
}

impl UserData for Text {
	fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
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
				_ => return Err("expected a WRAP or WRAP_TRIM or WRAP_OFF".into_lua_err()),
			};
			Ok(ud)
		});
		methods.add_method("max_width", |_, me, ()| {
			Ok(me.inner.lines.iter().take(me.area.height as usize).map(|l| l.width()).max())
		});
	}
}

impl Renderable for Text {
	fn area(&self) -> ratatui::layout::Rect { *self.area }

	fn render(self: Box<Self>, buf: &mut ratatui::buffer::Buffer) {
		let area = *self.area;
		let p: ratatui::widgets::Paragraph = (*self).into();
		p.render(area, buf);
	}

	fn clone_render(&self, buf: &mut ratatui::buffer::Buffer) { Box::new(self.clone()).render(buf) }
}
