use mlua::{AnyUserData, ExternalError, IntoLua, Lua, Table, UserData, Value};
use ratatui::widgets::Borders;

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
		let new = lua.create_function(|_, (_, area, direction): (Table, RectRef, u8)| {
			Ok(Self {
				area: *area,

				position: Borders::from_bits_truncate(direction),
				symbol:   Default::default(),
				style:    Default::default(),
			})
		})?;

		let bar = lua.create_table_from([
			("NONE", Borders::NONE.bits().into_lua(lua)?),
			("TOP", Borders::TOP.bits().into_lua(lua)?),
			("RIGHT", Borders::RIGHT.bits().into_lua(lua)?),
			("BOTTOM", Borders::BOTTOM.bits().into_lua(lua)?),
			("LEFT", Borders::LEFT.bits().into_lua(lua)?),
			("ALL", Borders::ALL.bits().into_lua(lua)?),
		])?;

		bar.set_metatable(Some(lua.create_table_from([("__call", new)])?));

		ui.set("Bar", bar)
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
