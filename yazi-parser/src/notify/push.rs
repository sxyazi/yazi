use mlua::{FromLua, IntoLua, Lua, Value};
use yazi_core::notify::MessageOpt;
use yazi_shared::event::{ActionCow, FromAction};

#[derive(Clone, Debug)]
pub struct PushForm {
	pub opt: MessageOpt,
}

impl From<MessageOpt> for PushForm {
	fn from(opt: MessageOpt) -> Self { Self { opt } }
}

impl<C> FromAction<C> for PushForm {
	fn from_action(mut a: ActionCow, cx: &C) -> anyhow::Result<Self> {
		Ok(Self {
			opt: if let Some(opt) = a.take_any("opt") { opt } else { FromAction::from_action(a, cx)? },
		})
	}
}

impl FromLua for PushForm {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
		Ok(Self { opt: MessageOpt::from_lua(value, lua)? })
	}
}

impl IntoLua for PushForm {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> { self.opt.into_lua(lua) }
}
