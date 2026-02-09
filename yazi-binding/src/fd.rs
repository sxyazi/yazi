use mlua::{IntoLuaMulti, UserData, UserDataMethods};
use tokio::io::AsyncWriteExt;

use crate::Error;

pub struct Fd(pub yazi_vfs::provider::RwFile);

impl UserData for Fd {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_async_method_mut("write_all", |lua, mut me, src: mlua::String| async move {
			match me.0.write_all(&*src.as_bytes()).await {
				Ok(()) => true.into_lua_multi(&lua),
				Err(e) => (false, Error::Io(e)).into_lua_multi(&lua),
			}
		});
		methods.add_async_method_mut("flush", |lua, mut me, ()| async move {
			match me.0.flush().await {
				Ok(()) => true.into_lua_multi(&lua),
				Err(e) => (false, Error::Io(e)).into_lua_multi(&lua),
			}
		});
	}
}
