use mlua::{IntoLuaMulti, LuaString, UserData, UserDataMethods, Value};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use yazi_binding::Error;

use crate::provider::RwFile;

impl UserData for RwFile {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_async_method_mut("flush", |lua, mut me, ()| async move {
			match me.flush().await {
				Ok(()) => true.into_lua_multi(&lua),
				Err(e) => (false, Error::Io(e)).into_lua_multi(&lua),
			}
		});
		methods.add_async_method_mut("read", |lua, mut me, len: usize| async move {
			let mut buf = vec![0; len];
			match me.read(&mut buf).await {
				Ok(n) => {
					buf.truncate(n);
					lua.create_external_string(buf)?.into_lua_multi(&lua)
				}
				Err(e) => (Value::Nil, Error::Io(e)).into_lua_multi(&lua),
			}
		});
		methods.add_async_method_mut("write_all", |lua, mut me, src: LuaString| async move {
			match me.write_all(&src.as_bytes()).await {
				Ok(()) => true.into_lua_multi(&lua),
				Err(e) => (false, Error::Io(e)).into_lua_multi(&lua),
			}
		});
	}
}
