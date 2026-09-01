use mlua::{Function, Lua};
use yazi_core::input::InputHistories;

use super::Utils;

impl Utils {
	pub(super) fn input_history_max(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|_, ()| Ok(InputHistories::MAX))
	}
}
