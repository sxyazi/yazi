use mlua::{FromLua, IntoLua, Lua, Value};
use yazi_core::notify::MessageOpt;
use yazi_shared::event::ActionCow;

#[derive(Clone, Debug)]
pub struct PushForm {
	pub opt: MessageOpt,
}

impl From<MessageOpt> for PushForm {
	fn from(opt: MessageOpt) -> Self { Self { opt } }
}

impl TryFrom<ActionCow> for PushForm {
	type Error = anyhow::Error;

	fn try_from(mut a: ActionCow) -> Result<Self, Self::Error> {
		Ok(Self { opt: if let Some(opt) = a.take_any("opt") { opt } else { a.try_into()? } })
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
