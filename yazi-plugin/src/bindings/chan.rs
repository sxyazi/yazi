use mlua::{ExternalError, IntoLuaMulti, UserData, Value};
use yazi_binding::Error;

pub struct MpscTx(pub tokio::sync::mpsc::Sender<Value>);
pub struct MpscRx(pub tokio::sync::mpsc::Receiver<Value>);

impl UserData for MpscTx {
	fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
		methods.add_async_method("send", |lua, me, value: Value| async move {
			match me.0.send(value).await {
				Ok(()) => true.into_lua_multi(&lua),
				Err(e) => (false, Error::Custom(e.to_string().into())).into_lua_multi(&lua),
			}
		});
	}
}

impl UserData for MpscRx {
	fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
		methods.add_async_method_mut("recv", |lua, mut me, ()| async move {
			match me.0.recv().await {
				Some(value) => (value, true).into_lua_multi(&lua),
				None => (Value::Nil, false).into_lua_multi(&lua),
			}
		});
	}
}

pub struct MpscUnboundedTx(pub tokio::sync::mpsc::UnboundedSender<Value>);
pub struct MpscUnboundedRx(pub tokio::sync::mpsc::UnboundedReceiver<Value>);

impl UserData for MpscUnboundedTx {
	fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
		methods.add_method("send", |lua, me, value: Value| match me.0.send(value) {
			Ok(()) => true.into_lua_multi(lua),
			Err(e) => (false, Error::Custom(e.to_string().into())).into_lua_multi(lua),
		});
	}
}

impl UserData for MpscUnboundedRx {
	fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
		methods.add_async_method_mut("recv", |lua, mut me, ()| async move {
			match me.0.recv().await {
				Some(value) => (value, true).into_lua_multi(&lua),
				None => (Value::Nil, false).into_lua_multi(&lua),
			}
		});
	}
}

pub struct OneshotTx(pub Option<tokio::sync::oneshot::Sender<Value>>);
pub struct OneshotRx(pub Option<tokio::sync::oneshot::Receiver<Value>>);

impl UserData for OneshotTx {
	fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
		methods.add_method_mut("send", |lua, me, value: Value| {
			let Some(tx) = me.0.take() else {
				return Err("Oneshot sender already used".into_lua_err());
			};
			match tx.send(value) {
				Ok(()) => true.into_lua_multi(lua),
				Err(_) => (false, Error::Custom("Oneshot receiver closed".into())).into_lua_multi(lua),
			}
		});
	}
}

impl UserData for OneshotRx {
	fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
		methods.add_async_method_mut("recv", |lua, mut me, ()| async move {
			let Some(rx) = me.0.take() else {
				return Err("Oneshot receiver already used".into_lua_err());
			};
			match rx.await {
				Ok(value) => value.into_lua_multi(&lua),
				Err(e) => (Value::Nil, Error::Custom(e.to_string().into())).into_lua_multi(&lua),
			}
		});
	}
}
