use mlua::{IntoLuaMulti, UserDataMethods};
use tokio::io::AsyncWriteExt;
use yazi_binding::HttpInventory;
use yazi_shared::url::UrlRef;
use yazi_shim::fs::Error;

use crate::engine::RwFile;

inventory::submit! {
	HttpInventory {
		register: |registry| {
			registry.add_async_method_once("write", |lua, me, url: UrlRef| async move {
				let result = match RwFile::create(&*url).await {
					Ok(mut file) => {
						let written = me.write(&mut file).await;
						written.and(file.shutdown().await)
					},
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
