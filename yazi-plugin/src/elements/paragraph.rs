use ansi_to_tui::IntoText;
use mlua::{AnyUserData, ExternalError, ExternalResult, IntoLua, Lua, Table, UserData, Value};
use ratatui::widgets::Widget;

use super::{Line, RectRef, Renderable, Style};

// Alignment
const LEFT: u8 = 0;
const CENTER: u8 = 1;
const RIGHT: u8 = 2;

// Wrap
const WRAP_NO: u8 = 0;
const WRAP: u8 = 1;
const WRAP_TRIM: u8 = 2;

#[derive(Clone, Debug, Default)]
pub struct Paragraph {
	pub area: ratatui::layout::Rect,

	pub text:      ratatui::text::Text<'static>,
	pub style:     Option<ratatui::style::Style>,
	pub alignment: ratatui::layout::Alignment,
	pub wrap:      u8,
}

impl Paragraph {
	pub fn install(lua: &Lua, ui: &Table) -> mlua::Result<()> {
		let new = lua.create_function(|_, (_, area, lines): (Table, RectRef, Vec<Line>)| {
			Ok(Paragraph {
				area: *area,
				text: lines.into_iter().map(|s| s.0).collect::<Vec<_>>().into(),
				..Default::default()
			})
		})?;

		let parse = lua.create_function(|_, (area, code): (RectRef, mlua::String)| {
			Ok(Paragraph { area: *area, text: code.into_text().into_lua_err()?, ..Default::default() })
		})?;

		let paragraph = lua.create_table_from([
			("parse", parse.into_lua(lua)?),
			// Alignment
			("LEFT", LEFT.into_lua(lua)?),
			("CENTER", CENTER.into_lua(lua)?),
			("RIGHT", RIGHT.into_lua(lua)?),
			// Wrap
			("WRAP_OFF", WRAP_NO.into_lua(lua)?),
			("WRAP", WRAP.into_lua(lua)?),
			("WRAP_TRIM", WRAP_TRIM.into_lua(lua)?),
		])?;

		paragraph.set_metatable(Some(lua.create_table_from([("__call", new)])?));

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
					Value::Table(tb) => me.style = Some(Style::from(tb).0),
					Value::UserData(ud) => me.style = Some(ud.borrow::<Style>()?.0),
					_ => return Err("expected a Style or Table or nil".into_lua_err()),
				}
			}
			Ok(ud)
		});
		methods.add_function("align", |_, (ud, align): (AnyUserData, u8)| {
			ud.borrow_mut::<Self>()?.alignment = match align {
				CENTER => ratatui::layout::Alignment::Center,
				RIGHT => ratatui::layout::Alignment::Right,
				_ => ratatui::layout::Alignment::Left,
			};
			Ok(ud)
		});
		methods.add_function("wrap", |_, (ud, wrap): (AnyUserData, u8)| {
			ud.borrow_mut::<Self>()?.wrap = match wrap {
				w @ (WRAP | WRAP_TRIM | WRAP_NO) => w,
				_ => return Err("expected a WRAP or WRAP_TRIM or WRAP_OFF".into_lua_err()),
			};
			Ok(ud)
		});
	}
}

impl Renderable for Paragraph {
	fn area(&self) -> ratatui::layout::Rect { self.area }

	fn render(self: Box<Self>, buf: &mut ratatui::buffer::Buffer) {
		let mut p = ratatui::widgets::Paragraph::new(self.text);
		if let Some(style) = self.style {
			p = p.style(style);
		}

		if self.wrap != WRAP_NO {
			p = p.wrap(ratatui::widgets::Wrap { trim: self.wrap == WRAP_TRIM });
		}

		p.alignment(self.alignment).render(self.area, buf);
	}

	fn clone_render(&self, buf: &mut ratatui::buffer::Buffer) { Box::new(self.clone()).render(buf) }
}
