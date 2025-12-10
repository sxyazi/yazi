use mlua::{MultiValue, UserData, UserDataMethods};
use tokio::task::JoinHandle;

pub enum Handle {
	AsyncFn(JoinHandle<mlua::Result<MultiValue>>),
}

impl UserData for Handle {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_method("abort", |_, me, ()| {
			Ok(match me {
				Self::AsyncFn(h) => h.abort(),
			})
		});
	}
}
