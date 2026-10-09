use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::{event::{ActionCow, FromAction}, url::UrlBuf};

#[derive(Debug, Default)]
pub struct UpdatePagedForm {
	pub page:    Option<usize>,
	pub only_if: Option<UrlBuf>,
}

impl<C> FromAction<C> for UpdatePagedForm {
	fn from_action(mut a: ActionCow, _: &C) -> anyhow::Result<Self> {
		Ok(Self { page: a.first().ok(), only_if: a.take("only-if").ok() })
	}
}

impl From<()> for UpdatePagedForm {
	fn from(_: ()) -> Self { Self::default() }
}

impl FromLua for UpdatePagedForm {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for UpdatePagedForm {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
