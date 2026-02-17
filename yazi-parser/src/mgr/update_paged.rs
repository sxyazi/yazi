use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::{event::ActionCow, url::UrlCow};

#[derive(Debug, Default)]
pub struct UpdatePagedOpt {
	pub page:    Option<usize>,
	pub only_if: Option<UrlCow<'static>>,
}

impl From<ActionCow> for UpdatePagedOpt {
	fn from(mut a: ActionCow) -> Self {
		Self { page: a.first().ok(), only_if: a.take("only-if").ok() }
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
