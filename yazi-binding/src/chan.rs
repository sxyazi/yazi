use mlua::{ExternalError, FromLua, IntoLua, IntoLuaMulti, UserData, Value};
use yazi_codegen::FromLuaOwned;

use crate::Error;

#[derive(FromLuaOwned)]
pub struct MpscTx<T: FromLua + 'static>(pub tokio::sync::mpsc::Sender<T>);
pub struct MpscRx<T: IntoLua + 'static>(pub tokio::sync::mpsc::Receiver<T>);

impl<T: FromLua> UserData for MpscTx<T> {
	fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
		methods.add_async_method("send", |lua, me, value: Value| async move {
			match me.0.send(T::from_lua(value, &lua)?).await {
				Ok(()) => true.into_lua_multi(&lua),
				Err(e) => (false, Error::custom(e.to_string())).into_lua_multi(&lua),
			}
		});
	}
}

impl<T: IntoLua + 'static> UserData for MpscRx<T> {
	fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
		methods.add_async_method_mut("recv", |lua, mut me, ()| async move {
			match me.0.recv().await {
				Some(value) => (value, true).into_lua_multi(&lua),
				None => (Value::Nil, false).into_lua_multi(&lua),
			}
		});
	}
}

#[derive(FromLuaOwned)]
pub struct MpscUnboundedTx<T: FromLua + 'static>(pub tokio::sync::mpsc::UnboundedSender<T>);
pub struct MpscUnboundedRx<T: IntoLua + 'static>(pub tokio::sync::mpsc::UnboundedReceiver<T>);

impl<T: FromLua> UserData for MpscUnboundedTx<T> {
	fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
		methods.add_method("send", |lua, me, value: Value| match me.0.send(T::from_lua(value, lua)?) {
			Ok(()) => true.into_lua_multi(lua),
			Err(e) => (false, Error::custom(e.to_string())).into_lua_multi(lua),
		});
	}
}

impl<T: IntoLua + 'static> UserData for MpscUnboundedRx<T> {
	fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
		methods.add_async_method_mut("recv", |lua, mut me, ()| async move {
			match me.0.recv().await {
				Some(value) => (value, true).into_lua_multi(&lua),
				None => (Value::Nil, false).into_lua_multi(&lua),
			}
		});
	}
}

#[derive(FromLuaOwned)]
pub struct OneshotTx<T: FromLua + 'static>(pub Option<tokio::sync::oneshot::Sender<T>>);
pub struct OneshotRx<T: IntoLua + 'static>(pub Option<tokio::sync::oneshot::Receiver<T>>);

impl<T: FromLua> UserData for OneshotTx<T> {
	fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
		methods.add_method_mut("send", |lua, me, value: Value| {
			let Some(tx) = me.0.take() else {
				return Err("Oneshot sender already used".into_lua_err());
			};
			match tx.send(T::from_lua(value, lua)?) {
				Ok(()) => true.into_lua_multi(lua),
				Err(_) => (false, Error::custom("Oneshot receiver closed")).into_lua_multi(lua),
			}
		});
	}
}

impl<T: IntoLua + 'static> UserData for OneshotRx<T> {
	fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
		methods.add_async_method_mut("recv", |lua, mut me, ()| async move {
			let Some(rx) = me.0.take() else {
				return Err("Oneshot receiver already used".into_lua_err());
			};
			match rx.await {
				Ok(value) => value.into_lua_multi(&lua),
				Err(e) => (Value::Nil, Error::custom(e.to_string())).into_lua_multi(&lua),
			}
		});
	}
}
