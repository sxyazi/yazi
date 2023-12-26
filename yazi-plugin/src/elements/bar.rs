use mlua::{AnyUserData, ExternalError, Lua, Table, UserData, Value};

use super::{RectRef, Renderable, Style};

#[derive(Clone)]
pub struct Bar {
	area: ratatui::layout::Rect,

	position: ratatui::widgets::Borders,
	symbol:   String,
	style:    Option<ratatui::style::Style>,
}

impl Bar {
	pub fn install(lua: &Lua, ui: &Table) -> mlua::Result<()> {
		ui.set(
			"Bar",
			lua.create_function(|_, (area, direction): (RectRef, u8)| {
				Ok(Self {
					area: *area,

					position: match direction {
						1 => ratatui::widgets::Borders::TOP,
						2 => ratatui::widgets::Borders::RIGHT,
						4 => ratatui::widgets::Borders::BOTTOM,
						8 => ratatui::widgets::Borders::LEFT,
						_ => ratatui::widgets::Borders::NONE,
					},
					symbol:   Default::default(),
					style:    Default::default(),
				})
			})?,
		)
	}
}

impl UserData for Bar {
	fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
		methods.add_function("symbol", |_, (ud, symbol): (AnyUserData, String)| {
			ud.borrow_mut::<Self>()?.symbol = symbol;
			Ok(ud)
		});
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
	}
}

impl Renderable for Bar {
	fn render(self: Box<Self>, buf: &mut ratatui::buffer::Buffer) {
		use ratatui::widgets::Borders;
		if self.area.area() == 0 {
			return;
		}

		let symbol = if !self.symbol.is_empty() {
			&self.symbol
		} else if self.position.intersects(Borders::TOP | Borders::BOTTOM) {
			"─"
		} else if self.position.intersects(Borders::LEFT | Borders::RIGHT) {
			"│"
		} else {
			" "
		};

		if self.position.contains(Borders::LEFT) {
			for y in self.area.top()..self.area.bottom() {
				let cell = buf.get_mut(self.area.left(), y).set_symbol(symbol);
				if let Some(style) = self.style {
					cell.set_style(style);
				}
			}
		}
		if self.position.contains(Borders::TOP) {
			for x in self.area.left()..self.area.right() {
				let cell = buf.get_mut(x, self.area.top()).set_symbol(symbol);
				if let Some(style) = self.style {
					cell.set_style(style);
				}
			}
		}
		if self.position.contains(Borders::RIGHT) {
			let x = self.area.right() - 1;
			for y in self.area.top()..self.area.bottom() {
				let cell = buf.get_mut(x, y).set_symbol(symbol);
				if let Some(style) = self.style {
					cell.set_style(style);
				}
			}
		}
		if self.position.contains(Borders::BOTTOM) {
			let y = self.area.bottom() - 1;
			for x in self.area.left()..self.area.right() {
				let cell = buf.get_mut(x, y).set_symbol(symbol);
				if let Some(style) = self.style {
					cell.set_style(style);
				}
			}
		}
	}

	fn clone_render(&self, buf: &mut ratatui::buffer::Buffer) { Box::new(self.clone()).render(buf) }
}
