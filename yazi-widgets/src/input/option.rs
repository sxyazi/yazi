use mlua::{ExternalError, FromLua, Lua, Table, Value};
use yazi_binding::position::Position;
use yazi_macro::impl_data_any;

use crate::input::{InputCallback, InputStyles};

#[derive(Clone, Debug, Default)]
pub struct InputOpt {
	pub name:       String,
	pub title:      String,
	pub value:      String,
	pub history:    String,
	pub styles:     InputStyles,
	pub cursor:     Option<usize>,
	pub obscure:    bool,
	pub blinking:   bool,
	pub position:   Position,
	pub realtime:   bool,
	pub completion: bool,
	pub cb:         Option<Box<dyn InputCallback>>,
}

impl_data_any!(InputOpt);

impl InputOpt {
	pub fn with_value(mut self, value: impl Into<String>) -> Self {
		self.value = value.into();
		self
	}

	pub fn with_cursor(mut self, cursor: Option<usize>) -> Self {
		self.cursor = cursor;
		self
	}

	pub fn with_cb(mut self, cb: impl InputCallback) -> Self {
		self.cb = Some(Box::new(cb));
		self
	}
}

impl TryFrom<&Table> for InputOpt {
	type Error = mlua::Error;

	fn try_from(t: &Table) -> Result<Self, Self::Error> {
		Ok(Self {
			name:       t.raw_get("name").unwrap_or_default(),
			title:      t.raw_get("title").unwrap_or_default(),
			value:      t.raw_get("value").unwrap_or_default(),
			history:    t.raw_get("history").unwrap_or_default(),
			styles:     t.raw_get("styles")?,
			cursor:     None,
			obscure:    t.raw_get("obscure")?,
			blinking:   false,
			position:   t.raw_get::<Position>("pos").unwrap_or_default().with_height(3),
			realtime:   t.raw_get("realtime")?,
			completion: false,
			cb:         None,
		})
	}
}

impl FromLua for InputOpt {
	fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
		match value {
			Value::Table(tb) => Self::try_from(&tb),
			_ => Err("expected a table".into_lua_err()),
		}
	}
}
