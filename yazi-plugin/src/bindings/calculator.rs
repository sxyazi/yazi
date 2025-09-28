use mlua::{IntoLuaMulti, UserData, UserDataMethods, Value};
use yazi_binding::Error;

pub enum SizeCalculator {
	Local(yazi_fs::provider::local::SizeCalculator),
	Remote(yazi_vfs::provider::SizeCalculator),
}

impl UserData for SizeCalculator {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_async_method_mut("recv", |lua, mut me, ()| async move {
			let next = match &mut *me {
				Self::Local(it) => it.next().await,
				Self::Remote(it) => it.next().await,
			};

			match next {
				Ok(value) => value.into_lua_multi(&lua),
				Err(e) => (Value::Nil, Error::Io(e)).into_lua_multi(&lua),
			}
		});
	}
}
