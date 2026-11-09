use mlua::{IntoLuaMulti, UserDataMethods};
use tokio::io::AsyncWriteExt;
use yazi_binding::HttpInventory;
use yazi_shim::fs::Error;

use crate::engine::RwFileRef;

inventory::submit! {
	HttpInventory {
		register: |registry| {
			registry.add_async_method_once("write", |lua, me, mut file: RwFileRef| async move {
				let written = me.write(&mut *file).await;
				match written.and(file.flush().await) {
					Ok(()) => true.into_lua_multi(&lua),
					Err(e) => (false, Error::from(e)).into_lua_multi(&lua),
				}
			});
		},
	}
}
