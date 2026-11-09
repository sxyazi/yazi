use mlua::{IntoLuaMulti, MetaMethod, UserData, UserDataFields, UserDataMethods, Value};
use yazi_shim::fs::Error;

pub enum SizeCalculator {
	Local(yazi_fs::engine::local::SizeCalculator),
	Virtual(yazi_vfs::engine::SizeCalculator),
}

impl UserData for SizeCalculator {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("lstat", |_, me| {
			Ok(match me {
				Self::Local(c) => c.lstat(),
				Self::Virtual(c) => c.lstat(),
			})
		});
	}

	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_async_meta_method_mut(MetaMethod::Call, |lua, mut me, ()| async move {
			let next = match &mut *me {
				Self::Local(c) => c.next().await,
				Self::Virtual(c) => c.next().await,
			};

			match next {
				Ok(next) => (next, Value::Nil).into_lua_multi(&lua),
				Err(e) => (false, Error::from(e)).into_lua_multi(&lua),
			}
		});
	}
}
