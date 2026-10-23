use std::sync::Arc;

use arc_swap::Guard;
use compact_str::CompactString;
use mlua::{FromLua, Lua, LuaSerdeExt, Table, UserData, Value};
use ratatui_core::symbols::border;
use serde::{Deserialize, Deserializer, de};
use unicode_width::UnicodeWidthStr;
use yazi_shim::mlua::LuaTableExt;

use super::BorderType;

#[serde_with::apply(
	CompactString => #[serde(default, deserialize_with = "BorderSet::deserialize_symbol")]
)]
#[derive(Clone, Debug, Default, Deserialize)]
pub struct BorderSet {
	#[serde(rename = "as", alias = "0")]
	r#as: BorderType,

	tl: CompactString,
	tc: CompactString,
	tr: CompactString,
	cl: CompactString,
	cr: CompactString,
	bl: CompactString,
	bc: CompactString,
	br: CompactString,
}

impl From<BorderType> for BorderSet {
	fn from(r#as: BorderType) -> Self { Self { r#as, ..Default::default() } }
}

impl From<Guard<Arc<BorderSet>>> for BorderSet {
	fn from(value: Guard<Arc<Self>>) -> Self { Self::clone(&value) }
}

impl TryFrom<Table> for BorderSet {
	type Error = mlua::Error;

	fn try_from(t: Table) -> Result<Self, Self::Error> { t.deserialize() }
}

impl BorderSet {
	pub fn as_set(&self) -> border::Set<'_> {
		let mut set = self.r#as.to_border_set();
		for (symbol, target) in [
			(&self.tl, &mut set.top_left),
			(&self.tc, &mut set.horizontal_top),
			(&self.tr, &mut set.top_right),
			(&self.cl, &mut set.vertical_left),
			(&self.cr, &mut set.vertical_right),
			(&self.bl, &mut set.bottom_left),
			(&self.bc, &mut set.horizontal_bottom),
			(&self.br, &mut set.bottom_right),
		] {
			if !symbol.is_empty() {
				*target = symbol;
			}
		}

		set
	}

	fn deserialize_symbol<'de, D: Deserializer<'de>>(de: D) -> Result<CompactString, D::Error> {
		let s = CompactString::deserialize(de)?;
		if s.chars().any(char::is_control) || (!s.is_empty() && s.width() != 1) {
			return Err(de::Error::custom("expected an empty or 1-width border symbol"));
		}

		Ok(s)
	}
}

impl FromLua for BorderSet {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
		match value {
			Value::String(_) => Ok(lua.from_value::<BorderType>(value)?.into()),
			Value::UserData(ud) => Ok(ud.borrow::<Self>()?.clone()),
			Value::Table(t) => t.try_into(),
			value => lua.from_value(value),
		}
	}
}

impl UserData for BorderSet {}
