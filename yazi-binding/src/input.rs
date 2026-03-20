use std::pin::Pin;

use mlua::{UserData, UserDataMethods};
use tokio::pin;
use tokio_stream::StreamExt;
use yazi_widgets::input::InputEvent;

pub struct InputRx<T: StreamExt<Item = InputEvent>> {
	inner: T,
}

impl<T: StreamExt<Item = InputEvent>> InputRx<T> {
	pub fn new(inner: T) -> Self { Self { inner } }

	pub async fn consume(inner: T) -> (Option<String>, u8) {
		pin!(inner);
		inner.next().await.map(Self::parse).unwrap_or((None, 0))
	}

	fn parse(res: InputEvent) -> (Option<String>, u8) {
		match res {
			InputEvent::Submit(s) => (Some(s), 1),
			InputEvent::Cancel(s) => (Some(s), 2),
			InputEvent::Type(s) => (Some(s), 3),
			_ => (None, 0),
		}
	}
}

impl<T: StreamExt<Item = InputEvent> + 'static> UserData for InputRx<T> {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_async_method_mut("recv", |_, mut me, ()| async move {
			let mut inner = unsafe { Pin::new_unchecked(&mut me.inner) };
			Ok(inner.next().await.map(Self::parse).unwrap_or((None, 0)))
		});
	}
}
