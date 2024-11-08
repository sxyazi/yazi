use mlua::{AnyUserData, Lua, Table, UserData};
use ratatui::widgets::Borders;

use super::{Rect, Renderable};

#[derive(Clone, Default)]
pub struct Bar {
	area: Rect,

	direction: ratatui::widgets::Borders,
	symbol:    String,
	style:     ratatui::style::Style,
}

impl Bar {
	pub fn install(lua: &Lua, ui: &Table) -> mlua::Result<()> {
		let new = lua.create_function(|_, (_, direction): (Table, u8)| {
			Ok(Self { direction: Borders::from_bits_truncate(direction), ..Default::default() })
		})?;

		let bar = lua.create_table_from([
			// Direction
			("NONE", Borders::NONE.bits()),
			("TOP", Borders::TOP.bits()),
			("RIGHT", Borders::RIGHT.bits()),
			("BOTTOM", Borders::BOTTOM.bits()),
			("LEFT", Borders::LEFT.bits()),
			("ALL", Borders::ALL.bits()),
		])?;

		bar.set_metatable(Some(lua.create_table_from([("__call", new)])?));

		ui.raw_set("Bar", bar)
	}
}

impl UserData for Bar {
	fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
		crate::impl_area_method!(methods);
		crate::impl_style_method!(methods, style);

		methods.add_function_mut("direction", |_, (ud, symbol): (AnyUserData, u8)| {
			ud.borrow_mut::<Self>()?.direction = Borders::from_bits_truncate(symbol);
			Ok(ud)
		});
		methods.add_function_mut("symbol", |_, (ud, symbol): (AnyUserData, String)| {
			ud.borrow_mut::<Self>()?.symbol = symbol;
			Ok(ud)
		});
	}
}

impl Renderable for Bar {
	fn area(&self) -> ratatui::layout::Rect { *self.area }

	fn render(self: Box<Self>, buf: &mut ratatui::buffer::Buffer) {
		if self.area.area() == 0 {
			return;
		}

		let symbol = if !self.symbol.is_empty() {
			&self.symbol
		} else if self.direction.intersects(Borders::TOP | Borders::BOTTOM) {
			"─"
		} else if self.direction.intersects(Borders::LEFT | Borders::RIGHT) {
			"│"
		} else {
			" "
		};

		if self.direction.contains(Borders::LEFT) {
			for y in self.area.top()..self.area.bottom() {
				buf[(self.area.left(), y)].set_style(self.style).set_symbol(symbol);
			}
		}
		if self.direction.contains(Borders::TOP) {
			for x in self.area.left()..self.area.right() {
				buf[(x, self.area.top())].set_style(self.style).set_symbol(symbol);
			}
		}
		if self.direction.contains(Borders::RIGHT) {
			let x = self.area.right() - 1;
			for y in self.area.top()..self.area.bottom() {
				buf[(x, y)].set_style(self.style).set_symbol(symbol);
			}
		}
		if self.direction.contains(Borders::BOTTOM) {
			let y = self.area.bottom() - 1;
			for x in self.area.left()..self.area.right() {
				buf[(x, y)].set_style(self.style).set_symbol(symbol);
			}
		}
	}

	fn clone_render(&self, buf: &mut ratatui::buffer::Buffer) { Box::new(self.clone()).render(buf) }
}
