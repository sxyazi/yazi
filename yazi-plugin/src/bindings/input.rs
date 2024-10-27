use std::pin::Pin;

use mlua::{UserData, prelude::LuaUserDataMethods};
use tokio::pin;
use tokio_stream::StreamExt;
use yazi_shared::errors::InputError;

pub struct InputRx<T: StreamExt<Item = Result<String, InputError>>> {
	inner: T,
}

impl<T: StreamExt<Item = Result<String, InputError>>> InputRx<T> {
	pub fn new(inner: T) -> Self { Self { inner } }

	pub async fn consume(inner: T) -> (Option<String>, u8) {
		pin!(inner);
		inner.next().await.map(Self::parse).unwrap_or((None, 0))
	}

	fn parse(res: Result<String, InputError>) -> (Option<String>, u8) {
		match res {
			Ok(s) => (Some(s), 1),
			Err(InputError::Canceled(s)) => (Some(s), 2),
			Err(InputError::Typed(s)) => (Some(s), 3),
			_ => (None, 0),
		}
	}
}

impl<T: StreamExt<Item = Result<String, InputError>> + 'static> UserData for InputRx<T> {
	fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
		methods.add_async_method_mut("recv", |_, mut me, ()| async move {
			let mut inner = unsafe { Pin::new_unchecked(&mut me.inner) };
			Ok(inner.next().await.map(Self::parse).unwrap_or((None, 0)))
		});
	}
}
