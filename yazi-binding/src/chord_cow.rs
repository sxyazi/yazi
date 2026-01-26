use mlua::UserData;
use yazi_codegen::FromLuaOwned;

#[derive(Clone, FromLuaOwned)]
pub struct ChordCow(pub yazi_config::keymap::ChordCow);

impl From<ChordCow> for yazi_config::keymap::ChordCow {
	fn from(value: ChordCow) -> Self { value.0 }
}

impl UserData for ChordCow {}
