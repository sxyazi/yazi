use mlua::{ExternalError, IntoLua, Lua, MetaMethod, Table, UserData, Value};
use ratatui::widgets::Widget;

use super::{Area, Text};

const EXPECTED: &str = "expected a table of strings, Texts, Lines or Spans";

// --- List
#[derive(Clone, Debug, Default)]
pub struct List {
	area: Area,

	inner: ratatui::widgets::List<'static>,
}

impl List {
	pub fn compose(lua: &Lua) -> mlua::Result<Value> {
		let new = lua.create_function(|_, (_, seq): (Table, Table)| {
			let mut items = Vec::with_capacity(seq.raw_len());
			for v in seq.sequence_values::<Value>() {
				items.push(Text::try_from(v?).map_err(|_| EXPECTED.into_lua_err())?);
			}

			Ok(Self { inner: ratatui::widgets::List::new(items), ..Default::default() })
		})?;

		let list = lua.create_table()?;
		list.set_metatable(Some(lua.create_table_from([(MetaMethod::Call.name(), new)])?));

		list.into_lua(lua)
	}

	pub(super) fn render(
		self,
		buf: &mut ratatui::buffer::Buffer,
		trans: impl FnOnce(yazi_config::popup::Position) -> ratatui::layout::Rect,
	) {
		self.inner.render(self.area.transform(trans), buf);
	}
}

impl UserData for List {
	fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
		crate::impl_area_method!(methods);
	}
}
