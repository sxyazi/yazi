use mlua::{prelude::LuaUserDataMethods, UserData};
use tokio::sync::mpsc::UnboundedReceiver;
use yazi_shared::InputError;

pub struct InputRx {
	inner: UnboundedReceiver<Result<String, InputError>>,
}

impl InputRx {
	pub fn new(inner: UnboundedReceiver<Result<String, InputError>>) -> Self { Self { inner } }
}

impl UserData for InputRx {
	fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
		methods.add_async_method_mut("recv", |_, me, ()| async move {
			let Some(res) = me.inner.recv().await else {
				return Ok((None, 0));
			};

			Ok(match res {
				Ok(s) => (Some(s), 1),
				Err(InputError::Typed(s)) => (Some(s), 2),
				_ => (None, 0),
			})
		});
	}
}
