use mlua::{AnyUserData, IntoLua, Lua, MetaMethod, Table, UserData, Value};
use ratatui::widgets::Borders;

use super::{Area, Edge};

#[derive(Clone, Debug, Default)]
pub struct Bar {
	area: Area,

	edge:   Edge,
	symbol: String,
	style:  ratatui::style::Style,
}

impl Bar {
	pub fn compose(lua: &Lua) -> mlua::Result<Value> {
		let new =
			lua.create_function(|_, (_, edge): (Table, Edge)| Ok(Self { edge, ..Default::default() }))?;

		let bar = lua.create_table()?;
		bar.set_metatable(Some(lua.create_table_from([(MetaMethod::Call.name(), new)])?));
		bar.into_lua(lua)
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
		} else if self.edge.intersects(Borders::TOP | Borders::BOTTOM) {
			"─"
		} else if self.edge.intersects(Borders::LEFT | Borders::RIGHT) {
			"│"
		} else {
			" "
		};

		if self.edge.contains(Borders::LEFT) {
			for y in rect.top()..rect.bottom() {
				buf[(rect.left(), y)].set_style(self.style).set_symbol(symbol);
			}
		}
		if self.edge.contains(Borders::TOP) {
			for x in rect.left()..rect.right() {
				buf[(x, rect.top())].set_style(self.style).set_symbol(symbol);
			}
		}
		if self.edge.contains(Borders::RIGHT) {
			let x = rect.right() - 1;
			for y in rect.top()..rect.bottom() {
				buf[(x, y)].set_style(self.style).set_symbol(symbol);
			}
		}
		if self.edge.contains(Borders::BOTTOM) {
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

		methods.add_function_mut("edge", |_, (ud, edge): (AnyUserData, Edge)| {
			ud.borrow_mut::<Self>()?.edge = edge;
			Ok(ud)
		});
		methods.add_function_mut("symbol", |_, (ud, symbol): (AnyUserData, String)| {
			ud.borrow_mut::<Self>()?.symbol = symbol;
			Ok(ud)
		});
	}
}
