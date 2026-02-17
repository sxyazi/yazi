use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_fs::FilterCase;
use yazi_shared::{SStr, event::ActionCow};

#[derive(Clone, Debug, Default)]
pub struct FilterOpt {
	pub query: SStr,
	pub case:  FilterCase,
	pub done:  bool,
}

impl TryFrom<ActionCow> for FilterOpt {
	type Error = anyhow::Error;

	fn try_from(mut a: ActionCow) -> Result<Self, Self::Error> {
		if let Some(opt) = a.take_any2("opt") {
			return opt;
		}

		Ok(Self {
			query: a.take_first().unwrap_or_default(),
			case:  FilterCase::from(&*a),
			done:  a.bool("done"),
		})
	}
}

impl FromLua for FilterOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for FilterOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
