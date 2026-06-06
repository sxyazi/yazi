use mlua::UserData;
use yazi_codegen::FromLuaOwned;

#[derive(FromLuaOwned)]
pub struct DataAny(Box<dyn yazi_shared::data::DataAny>);

impl DataAny {
	pub fn new<T>(value: T) -> Self
	where
		T: yazi_shared::data::DataAny,
	{
		Self(Box::new(value))
	}
}

impl UserData for DataAny {}
