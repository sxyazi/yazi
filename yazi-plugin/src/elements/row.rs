use mlua::{AnyUserData, ExternalError, IntoLua, Lua, MetaMethod, Table, UserData, Value};

use super::Cell;

const EXPECTED: &str = "expected a Row";

#[derive(Clone, Debug, Default)]
pub struct Row {
	pub(super) cells: Vec<Cell>,
	height:           u16,
	top_margin:       u16,
	bottom_margin:    u16,
	style:            ratatui::style::Style,
}

impl Row {
	pub fn compose(lua: &Lua) -> mlua::Result<Value> {
		let new = lua.create_function(|_, (_, cells): (Table, Vec<Cell>)| {
			Ok(Self { cells, ..Default::default() })
		})?;

		let row = lua.create_table()?;
		row.set_metatable(Some(lua.create_table_from([(MetaMethod::Call.name(), new)])?));

		row.into_lua(lua)
	}
}

impl From<Row> for ratatui::widgets::Row<'static> {
	fn from(value: Row) -> Self {
		Self::new(value.cells)
			.height(value.height.max(1))
			.top_margin(value.top_margin)
			.bottom_margin(value.bottom_margin)
			.style(value.style)
	}
}

impl TryFrom<Value> for Row {
	type Error = mlua::Error;

	fn try_from(value: Value) -> Result<Self, Self::Error> {
		Ok(match value {
			Value::UserData(ud) => {
				if let Ok(row) = ud.take() {
					row
				} else {
					Err(EXPECTED.into_lua_err())?
				}
			}
			_ => Err(EXPECTED.into_lua_err())?,
		})
	}
}

impl UserData for Row {
	fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
		crate::impl_style_method!(methods, style);

		methods.add_function_mut("height", |_, (ud, value): (AnyUserData, u16)| {
			ud.borrow_mut::<Self>()?.height = value;
			Ok(ud)
		});
		methods.add_function_mut("margin_t", |_, (ud, value): (AnyUserData, u16)| {
			ud.borrow_mut::<Self>()?.top_margin = value;
			Ok(ud)
		});
		methods.add_function_mut("margin_b", |_, (ud, value): (AnyUserData, u16)| {
			ud.borrow_mut::<Self>()?.bottom_margin = value;
			Ok(ud)
		});
	}
}
