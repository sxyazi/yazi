use mlua::{UserData, UserDataFields, UserDataMethods};
use yazi_shim::fs::Error;

pub enum SizeCalculator {
	Local(yazi_fs::engine::local::SizeCalculator),
	Virtual(yazi_vfs::engine::SizeCalculator),
}

impl UserData for SizeCalculator {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("stat", |_, me| {
			Ok(match me {
				Self::Local(c) => c.stat(),
				Self::Virtual(c) => c.stat(),
			})
		});
	}

	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_async_method_mut("recv", |_, mut me, ()| async move {
			let next = match &mut *me {
				Self::Local(c) => c.next().await,
				Self::Virtual(c) => c.next().await,
			};

			Ok(next.map_err(Error::from))
		});
	}
}
