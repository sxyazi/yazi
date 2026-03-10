use mlua::{IntoLuaMulti, UserData, UserDataFields, UserDataMethods, Value};

use crate::{Cha, Error};

pub enum SizeCalculator {
	Local(yazi_fs::provider::local::SizeCalculator),
	Remote(yazi_vfs::provider::SizeCalculator),
}

impl UserData for SizeCalculator {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("cha", |_, me| {
			Ok(Cha(match me {
				Self::Local(c) => c.cha(),
				Self::Remote(c) => c.cha(),
			}))
		});
	}

	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_async_method_mut("recv", |lua, mut me, ()| async move {
			let next = match &mut *me {
				Self::Local(c) => c.next().await,
				Self::Remote(c) => c.next().await,
			};

			match next {
				Ok(value) => value.into_lua_multi(&lua),
				Err(e) => (Value::Nil, Error::Io(e)).into_lua_multi(&lua),
			}
		});
	}
}
