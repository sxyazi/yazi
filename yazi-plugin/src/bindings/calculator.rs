use mlua::{IntoLuaMulti, UserData, UserDataMethods, Value};
use yazi_binding::Error;

pub struct SizeCalculator(pub yazi_fs::SizeCalculator);

impl UserData for SizeCalculator {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_async_method_mut("recv", |lua, mut me, ()| async move {
			match me.0.next().await {
				Ok(value) => value.into_lua_multi(&lua),
				Err(e) => (Value::Nil, Error::Io(e)).into_lua_multi(&lua),
			}
		});
	}
}
