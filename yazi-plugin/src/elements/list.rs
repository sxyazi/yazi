use mlua::{ExternalError, Lua, Table, UserData, Value};
use ratatui::widgets::Widget;

use super::{Rect, Renderable, Text};

// --- List
#[derive(Clone, Default)]
pub struct List {
	area: Rect,

	inner: ratatui::widgets::List<'static>,
}

impl List {
	pub fn compose(lua: &Lua) -> mlua::Result<Table> {
		let new = lua.create_function(|_, (_, seq): (Table, Table)| {
			let mut items = Vec::with_capacity(seq.raw_len());
			for v in seq.sequence_values::<Value>() {
				match v? {
					Value::Table(_) => Err("Nested table not supported".into_lua_err())?,
					v => items.push(Text::try_from(v)?),
				}
			}

			Ok(Self { inner: ratatui::widgets::List::new(items), ..Default::default() })
		})?;

		let list = lua.create_table()?;
		list.set_metatable(Some(lua.create_table_from([("__call", new)])?));

		Ok(list)
	}
}

impl UserData for List {
	fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
		crate::impl_area_method!(methods);
	}
}

impl Renderable for List {
	fn area(&self) -> ratatui::layout::Rect { *self.area }

	fn render(self: Box<Self>, buf: &mut ratatui::buffer::Buffer) {
		self.inner.render(*self.area, buf);
	}

	fn clone_render(&self, buf: &mut ratatui::buffer::Buffer) { Box::new(self.clone()).render(buf) }
}
