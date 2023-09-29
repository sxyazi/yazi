use mlua::{AnyUserData, FromLua, Lua, Table, UserData, Value};
use ratatui::widgets::Widget;

use super::{Rect, Style};
use crate::{layout::Line, GLOBALS, LUA};

#[derive(Clone, Debug)]
pub struct Paragraph {
	area: ratatui::layout::Rect,

	text:      ratatui::text::Text<'static>,
	style:     Option<ratatui::style::Style>,
	alignment: ratatui::prelude::Alignment,
}

impl Paragraph {
	pub(crate) fn install() -> mlua::Result<()> {
		let ui: Table = GLOBALS.get("ui")?;
		ui.set(
			"Paragraph",
			LUA.create_function(|_, (area, lines): (Rect, Vec<Line>)| {
				Ok(Self {
					area: area.0,

					text:      lines.into_iter().map(|s| s.0).collect::<Vec<_>>().into(),
					style:     None,
					alignment: Default::default(),
				})
			})?,
		)
	}

	pub fn render(self, buf: &mut ratatui::buffer::Buffer) {
		let mut p = ratatui::widgets::Paragraph::new(self.text);
		if let Some(style) = self.style {
			p = p.style(style);
		}

		p = p.alignment(self.alignment);
		p.render(self.area, buf)
	}
}

impl<'lua> FromLua<'lua> for Paragraph {
	fn from_lua(value: Value<'lua>, _: &'lua Lua) -> mlua::Result<Self> {
		match value {
			Value::UserData(ud) => Ok(ud.borrow::<Self>()?.clone()),
			_ => Err(mlua::Error::FromLuaConversionError {
				from:    value.type_name(),
				to:      "Paragraph",
				message: Some("expected a Paragraph".to_string()),
			}),
		}
	}
}

impl UserData for Paragraph {
	fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
		methods.add_function("style", |_, (ud, style): (AnyUserData, Style)| {
			ud.borrow_mut::<Self>()?.style = Some(style.0);
			Ok(ud)
		});
		methods.add_function("align", |_, (ud, align): (AnyUserData, u8)| {
			ud.borrow_mut::<Self>()?.alignment = match align {
				1 => ratatui::prelude::Alignment::Center,
				2 => ratatui::prelude::Alignment::Right,
				_ => ratatui::prelude::Alignment::Left,
			};
			Ok(ud)
		});
	}
}
