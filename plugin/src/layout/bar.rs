use mlua::{AnyUserData, FromLua, Lua, Table, UserData, Value};

use super::{Rect, Style};
use crate::{GLOBALS, LUA};

#[derive(Clone)]
pub struct Bar {
	area: ratatui::layout::Rect,

	position: ratatui::widgets::Borders,
	symbol:   String,
	style:    Option<ratatui::style::Style>,
}

impl Bar {
	pub(crate) fn install() -> mlua::Result<()> {
		let ui: Table = GLOBALS.get("ui")?;
		ui.set(
			"Bar",
			LUA.create_function(|_, (area, direction): (Rect, u8)| {
				Ok(Self {
					area: area.0,

					position: match direction {
						1 => ratatui::widgets::Borders::TOP,
						2 => ratatui::widgets::Borders::RIGHT,
						3 => ratatui::widgets::Borders::BOTTOM,
						4 => ratatui::widgets::Borders::LEFT,
						_ => ratatui::widgets::Borders::NONE,
					},
					symbol:   Default::default(),
					style:    Default::default(),
				})
			})?,
		)
	}

	pub fn render(self, buf: &mut ratatui::buffer::Buffer) {
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
}

impl<'lua> FromLua<'lua> for Bar {
	fn from_lua(value: Value<'lua>, _: &'lua Lua) -> mlua::Result<Self> {
		match value {
			Value::UserData(ud) => Ok(ud.borrow::<Self>()?.clone()),
			_ => Err(mlua::Error::FromLuaConversionError {
				from:    value.type_name(),
				to:      "Bar",
				message: Some("expected a Bar".to_string()),
			}),
		}
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
					_ => return Err(mlua::Error::external("expected a Style or Table or nil")),
				}
			}
			Ok(ud)
		});
	}
}
