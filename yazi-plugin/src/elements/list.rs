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
	pub fn install(lua: &Lua, ui: &Table) -> mlua::Result<()> {
		ui.raw_set(
			"List",
			lua.create_function(|_, tb: Table| {
				let mut items = Vec::with_capacity(tb.raw_len());
				for v in tb.sequence_values::<Value>() {
					match v? {
						Value::Table(_) => Err("Nested table not supported".into_lua_err())?,
						v => items.push(Text::try_from(v)?),
					}
				}

				Ok(Self { inner: ratatui::widgets::List::new(items), ..Default::default() })
			})?,
		)
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
