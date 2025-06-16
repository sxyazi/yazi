use std::{any::TypeId, mem};

use ansi_to_tui::IntoText;
use mlua::{AnyUserData, ExternalError, ExternalResult, IntoLua, Lua, MetaMethod, Table, UserData, Value};
use ratatui::widgets::Widget;
use yazi_binding::Error;

use super::{Area, Line, Span, Wrap};
use crate::elements::Align;

const EXPECTED: &str = "expected a string, Line, Span, or a table of them";

#[derive(Clone, Debug, Default)]
pub struct Text {
	pub area: Area,

	// TODO: block
	pub inner:  ratatui::text::Text<'static>,
	pub wrap:   Wrap,
	pub scroll: ratatui::layout::Position,
}

impl Text {
	pub fn compose(lua: &Lua) -> mlua::Result<Value> {
		let new = lua.create_function(|_, (_, value): (Table, Value)| Text::try_from(value))?;

		let parse = lua.create_function(|_, code: mlua::String| {
			Ok(Text { inner: code.as_bytes().into_text().into_lua_err()?, ..Default::default() })
		})?;

		let text = lua.create_table_from([("parse", parse)])?;
		text.set_metatable(Some(lua.create_table_from([(MetaMethod::Call.name(), new)])?));
		text.into_lua(lua)
	}

	pub(super) fn render(
		self,
		buf: &mut ratatui::buffer::Buffer,
		trans: impl Fn(yazi_config::popup::Position) -> ratatui::layout::Rect,
	) {
		let rect = self.area.transform(trans);
		if self.wrap.is_none() && self.scroll == Default::default() {
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
			Value::UserData(ud) => match ud.type_id() {
				Some(t) if t == TypeId::of::<Line>() => ud.take::<Line>()?.inner.into(),
				Some(t) if t == TypeId::of::<Span>() => ud.take::<Span>()?.0.into(),
				Some(t) if t == TypeId::of::<Text>() => return ud.take(),
				Some(t) if t == TypeId::of::<Error>() => ud.take::<Error>()?.into_string().into(),
				_ => Err(EXPECTED.into_lua_err())?,
			},
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
		if let Some(wrap) = value.wrap.0 {
			p = p.wrap(wrap);
		}
		p.scroll((value.scroll.y, value.scroll.x))
	}
}

impl UserData for Text {
	fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
		crate::impl_area_method!(methods);
		crate::impl_style_method!(methods, inner.style);
		yazi_binding::impl_style_shorthands!(methods, inner.style);

		methods.add_function_mut("align", |_, (ud, align): (AnyUserData, Align)| {
			ud.borrow_mut::<Self>()?.inner.alignment = Some(align.0);
			Ok(ud)
		});
		methods.add_function_mut("wrap", |_, (ud, wrap): (AnyUserData, Wrap)| {
			ud.borrow_mut::<Self>()?.wrap = wrap;
			Ok(ud)
		});
		methods.add_function_mut("scroll", |_, (ud, x, y): (AnyUserData, u16, u16)| {
			ud.borrow_mut::<Self>()?.scroll = ratatui::layout::Position { x, y };
			Ok(ud)
		});
		methods.add_method("max_width", |_, me, ()| {
			Ok(me.inner.lines.iter().take(me.area.size().height as usize).map(|l| l.width()).max())
		});
	}
}
