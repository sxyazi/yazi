use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::{event::{CmdCow, Data}, url::Url};

#[derive(Debug, Default)]
pub struct UpdatePagedOpt {
	pub page:    Option<usize>,
	pub only_if: Option<Url>,
}

impl From<CmdCow> for UpdatePagedOpt {
	fn from(mut c: CmdCow) -> Self {
		Self { page: c.first().and_then(Data::as_usize), only_if: c.take_url("only-if") }
	}
}

impl From<()> for UpdatePagedOpt {
	fn from(_: ()) -> Self { Self::default() }
}

impl FromLua for UpdatePagedOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for UpdatePagedOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
