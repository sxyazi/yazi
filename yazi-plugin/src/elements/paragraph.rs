use ansi_to_tui::IntoText;
use mlua::{AnyUserData, ExternalError, ExternalResult, Lua, Table, UserData, Value};
use ratatui::widgets::Widget;

use super::{Line, RectRef, Renderable, Style};

#[derive(Clone, Debug, Default)]
pub struct Paragraph {
	pub area: ratatui::layout::Rect,

	pub text:      ratatui::text::Text<'static>,
	pub style:     Option<ratatui::style::Style>,
	pub alignment: ratatui::prelude::Alignment,
}

impl Paragraph {
	pub fn install(lua: &Lua, ui: &Table) -> mlua::Result<()> {
		#[inline]
		fn new(area: RectRef, lines: Vec<Line>) -> mlua::Result<Paragraph> {
			Ok(Paragraph {
				area: *area,
				text: lines.into_iter().map(|s| s.0).collect::<Vec<_>>().into(),
				..Default::default()
			})
		}

		#[inline]
		fn parse(area: RectRef, code: mlua::String) -> mlua::Result<Paragraph> {
			Ok(Paragraph { area: *area, text: code.into_text().into_lua_err()?, ..Default::default() })
		}

		let paragraph = lua.create_table_from([
			("new", lua.create_function(|_, (area, lines): (RectRef, Vec<Line>)| new(area, lines))?),
			("parse", lua.create_function(|_, (area, code): (RectRef, mlua::String)| parse(area, code))?),
		])?;

		paragraph.set_metatable(Some(lua.create_table_from([(
			"__call",
			lua.create_function(|_, (_, area, lines): (Table, RectRef, Vec<Line>)| new(area, lines))?,
		)])?));

		ui.set("Paragraph", paragraph)
	}
}

impl UserData for Paragraph {
	fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
		methods.add_function("style", |_, (ud, value): (AnyUserData, Value)| {
			{
				let mut me = ud.borrow_mut::<Self>()?;
				match value {
					Value::Nil => me.style = None,
					Value::Table(tbl) => me.style = Some(Style::from(tbl).0),
					Value::UserData(ud) => me.style = Some(ud.borrow::<Style>()?.0),
					_ => return Err("expected a Style or Table or nil".into_lua_err()),
				}
			}
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

impl Renderable for Paragraph {
	fn render(self: Box<Self>, buf: &mut ratatui::buffer::Buffer) {
		let mut p = ratatui::widgets::Paragraph::new(self.text);
		if let Some(style) = self.style {
			p = p.style(style);
		}

		p = p.alignment(self.alignment);
		p.render(self.area, buf)
	}

	fn clone_render(&self, buf: &mut ratatui::buffer::Buffer) { Box::new(self.clone()).render(buf) }
}
