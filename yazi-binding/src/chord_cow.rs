use mlua::UserData;
use yazi_codegen::FromLuaOwned;

#[derive(Clone, FromLuaOwned)]
pub struct ChordCow(pub yazi_config::keymap::ChordCow);

impl From<yazi_config::keymap::ChordCow> for ChordCow {
	fn from(value: yazi_config::keymap::ChordCow) -> Self { Self(value) }
}

impl From<ChordCow> for yazi_config::keymap::ChordCow {
	fn from(value: ChordCow) -> Self { value.0 }
}

impl UserData for ChordCow {}
