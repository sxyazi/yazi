use mlua::{IntoLuaMulti, MultiValue, UserData, UserDataMethods};
use yazi_shared::{data::Data, sendable::Sendable};
use yazi_shim::fs::Error;

use super::AsyncHandle;

pub enum Async {
	Async(AsyncHandle<MultiValue>),
	AsyncBlocking(AsyncHandle<Vec<Data>>),
}

impl UserData for Async {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_method("abort", |_, me, ()| {
			Ok(match me {
				Self::Async(h) => h.abort(),
				Self::AsyncBlocking(h) => h.abort(),
			})
		});
		methods.add_method("cancelled", |_, me, ()| {
			Ok(match me {
				Self::Async(h) => h.cancelled(),
				Self::AsyncBlocking(h) => h.cancelled(),
			})
		});
		methods.add_async_method_once("join", |lua, me, ()| async move {
			let result = match me {
				Self::Async(h) => h.join().await,
				Self::AsyncBlocking(h) => h
					.join()
					.await
					.and_then(|values| Sendable::list_to_values(&lua, values).map_err(Error::from_lua_err)),
			};

			match result {
				Ok(values) => (true, values).into_lua_multi(&lua),
				Err(error) => (false, error).into_lua_multi(&lua),
			}
		});
	}
}
