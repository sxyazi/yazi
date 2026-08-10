use mlua::{IntoLuaMulti, UserDataMethods};
use yazi_binding::HttpInventory;
use yazi_shared::url::UrlRef;
use yazi_shim::fs::Error;

use crate::engine;

inventory::submit! {
	HttpInventory {
		register: |registry| {
			registry.add_async_method_once("write", |lua, me, url: UrlRef| async move {
				let result = match engine::create(&*url).await {
					Ok(mut file) => me.write(&mut file).await,
					Err(e) => Err(e),
				};

				match result {
					Ok(()) => true.into_lua_multi(&lua),
					Err(e) => (false, Error::from(e)).into_lua_multi(&lua),
				}
			});
		},
	}
}
