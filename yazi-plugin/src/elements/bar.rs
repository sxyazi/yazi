use mlua::{Lua, MetaMethod, Table, UserData};
use ratatui::widgets::Borders;

use super::Area;

#[derive(Clone, Debug, Default)]
pub struct Bar {
	area: Area,

	direction: ratatui::widgets::Borders,
	symbol:    String,
	style:     ratatui::style::Style,
}

impl Bar {
	pub fn compose(lua: &Lua) -> mlua::Result<Table> {
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

		bar.set_metatable(Some(lua.create_table_from([(MetaMethod::Call.name(), new)])?));
		Ok(bar)
	}

	pub(super) fn render(
		self,
		buf: &mut ratatui::buffer::Buffer,
		trans: impl FnOnce(yazi_config::popup::Position) -> ratatui::layout::Rect,
	) {
		let rect = self.area.transform(trans);
		if rect.area() == 0 {
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
			for y in rect.top()..rect.bottom() {
				buf[(rect.left(), y)].set_style(self.style).set_symbol(symbol);
			}
		}
		if self.direction.contains(Borders::TOP) {
			for x in rect.left()..rect.right() {
				buf[(x, rect.top())].set_style(self.style).set_symbol(symbol);
			}
		}
		if self.direction.contains(Borders::RIGHT) {
			let x = rect.right() - 1;
			for y in rect.top()..rect.bottom() {
				buf[(x, y)].set_style(self.style).set_symbol(symbol);
			}
		}
		if self.direction.contains(Borders::BOTTOM) {
			let y = rect.bottom() - 1;
			for x in rect.left()..rect.right() {
				buf[(x, y)].set_style(self.style).set_symbol(symbol);
			}
		}
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
