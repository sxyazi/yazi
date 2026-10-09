use mlua::{FromLua, IntoLua, Lua, Value};
use yazi_core::which::WhichOpt;
use yazi_shared::event::{ActionCow, FromAction};

#[derive(Clone, Debug)]
pub struct ActivateForm {
	pub opt: WhichOpt,
}

impl From<WhichOpt> for ActivateForm {
	fn from(opt: WhichOpt) -> Self { Self { opt } }
}

impl<C> FromAction<C> for ActivateForm {
	fn from_action(mut a: ActionCow, cx: &C) -> anyhow::Result<Self> {
		Ok(Self {
			opt: if let Some(opt) = a.take_any("opt") { opt } else { FromAction::from_action(a, cx)? },
		})
	}
}

impl FromLua for ActivateForm {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
		Ok(Self { opt: WhichOpt::from_lua(value, lua)? })
	}
}

impl IntoLua for ActivateForm {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> { self.opt.into_lua(lua) }
}
