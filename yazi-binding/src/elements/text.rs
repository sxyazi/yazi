use std::{any::TypeId, mem};

use ansi_to_tui::IntoText;
use mlua::{AnyUserData, ExternalError, ExternalResult, FromLua, IntoLua, Lua, LuaString, MetaMethod, Table, UserData, UserDataMethods, Value};
use ratatui_core::widgets::Widget;
use yazi_shim::SStr;

use super::{Area, Line, Span, Wrap};
use crate::{Error, elements::{Align, Spatial}};

const EXPECTED: &str = "expected a string, Line, Span, or a table of them";

#[derive(Clone, Debug, Default)]
pub struct Text {
	area: Area,

	// TODO: block
	pub inner:  ratatui_core::text::Text<'static>,
	pub wrap:   Wrap,
	pub scroll: ratatui_core::layout::Position,
}

impl Text {
	pub fn compose(lua: &Lua) -> mlua::Result<Value> {
		let new = lua.create_function(|_, (_, text): (Table, Self)| Ok(text))?;

		let parse = lua.create_function(|_, code: LuaString| {
			Ok(Self { inner: code.as_bytes().into_text().into_lua_err()?, ..Default::default() })
		})?;

		let text = lua.create_table_from([("parse", parse)])?;
		text.set_metatable(Some(lua.create_table_from([(MetaMethod::Call.name(), new)])?))?;
		text.into_lua(lua)
	}

	pub fn wrap(mut self, wrap: impl Into<Wrap>) -> Self {
		self.wrap = wrap.into();
		self
	}
}

impl From<ratatui_core::text::Text<'static>> for Text {
	fn from(inner: ratatui_core::text::Text<'static>) -> Self { Self { inner, ..Default::default() } }
}

impl From<SStr> for Text {
	fn from(value: SStr) -> Self { Self { inner: value.into(), ..Default::default() } }
}

impl TryFrom<Table> for Text {
	type Error = mlua::Error;

	fn try_from(tb: Table) -> Result<Self, Self::Error> {
		let mut lines = Vec::with_capacity(tb.raw_len());
		for v in tb.sequence_values() {
			lines.push(match v? {
				Value::String(s) => s.to_string_lossy().into(),
				Value::UserData(ud) => match ud.type_id() {
					Some(t) if t == TypeId::of::<Span>() => ud.take::<Span>()?.0.into(),
					Some(t) if t == TypeId::of::<Line>() => ud.take::<Line>()?.inner,
					Some(t) if t == TypeId::of::<Error>() => ud.take::<Error>()?.into_string().into(),
					_ => Err(EXPECTED.into_lua_err())?,
				},
				_ => Err(EXPECTED.into_lua_err())?,
			})
		}
		Ok(Self { inner: lines.into(), ..Default::default() })
	}
}

impl From<Text> for ratatui_core::text::Text<'static> {
	fn from(value: Text) -> Self { value.inner }
}

impl From<Text> for ratatui_widgets::paragraph::Paragraph<'static> {
	fn from(mut value: Text) -> Self {
		let align = value.inner.alignment.take();
		let style = mem::take(&mut value.inner.style);

		let mut p = ratatui_widgets::paragraph::Paragraph::new(value.inner).style(style);
		if let Some(align) = align {
			p = p.alignment(align);
		}
		if let Some(wrap) = value.wrap.0 {
			p = p.wrap(wrap);
		}
		p.scroll((value.scroll.y, value.scroll.x))
	}
}

impl Spatial for Text {
	fn area(&self) -> Area { self.area }

	fn set_area(&mut self, area: Area) { self.area = area; }
}

impl Widget for Text {
	fn render(self, rect: ratatui_core::layout::Rect, buf: &mut ratatui_core::buffer::Buffer)
	where
		Self: Sized,
	{
		if self.wrap.is_none() && self.scroll == Default::default() {
			self.inner.render(rect, buf);
		} else {
			ratatui_widgets::paragraph::Paragraph::from(self).render(rect, buf);
		}
	}
}

impl Widget for &Text {
	fn render(self, rect: ratatui_core::layout::Rect, buf: &mut ratatui_core::buffer::Buffer)
	where
		Self: Sized,
	{
		if self.wrap.is_none() && self.scroll == Default::default() {
			(&self.inner).render(rect, buf);
		} else {
			ratatui_widgets::paragraph::Paragraph::from(self.clone()).render(rect, buf);
		}
	}
}

impl TryFrom<&AnyUserData> for Text {
	type Error = mlua::Error;

	fn try_from(value: &AnyUserData) -> Result<Self, Self::Error> {
		let inner = match value.type_id() {
			Some(t) if t == TypeId::of::<Self>() => return value.take(),
			Some(t) if t == TypeId::of::<Line>() => value.take::<Line>()?.inner.into(),
			Some(t) if t == TypeId::of::<Span>() => value.take::<Span>()?.0.into(),
			Some(t) if t == TypeId::of::<Error>() => value.take::<Error>()?.into_string().into(),
			_ => Err(EXPECTED.into_lua_err())?,
		};

		Ok(Self { inner, ..Default::default() })
	}
}

impl FromLua for Text {
	fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
		Ok(match value {
			Value::Table(tb) => Self::try_from(tb)?,
			Value::String(s) => Self { inner: s.to_string_lossy().into(), ..Default::default() },
			Value::UserData(ud) => Self::try_from(&ud)?,
			_ => Err(EXPECTED.into_lua_err())?,
		})
	}
}

impl UserData for Text {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		crate::impl_area_method!(methods);
		crate::impl_style_method!(methods, inner.style);
		crate::impl_style_shorthands!(methods, inner.style);

		methods.add_function("align", |_, (ud, align): (AnyUserData, Align)| {
			ud.borrow_mut::<Self>()?.inner.alignment = Some(align.0);
			Ok(ud)
		});
		methods.add_function("wrap", |_, (ud, wrap): (AnyUserData, Wrap)| {
			ud.borrow_mut::<Self>()?.wrap = wrap;
			Ok(ud)
		});
		methods.add_function("scroll", |_, (ud, x, y): (AnyUserData, u16, u16)| {
			ud.borrow_mut::<Self>()?.scroll = ratatui_core::layout::Position { x, y };
			Ok(ud)
		});
		methods.add_method("max_width", |_, me, ()| {
			Ok(me.inner.lines.iter().take(me.area.size().height as usize).map(|l| l.width()).max())
		});
	}
}
