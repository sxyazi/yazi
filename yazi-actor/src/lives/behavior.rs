use std::ops::Deref;

use mlua::{AnyUserData, UserData, UserDataMethods};

use super::{Lives, PtrCell};

pub(super) struct Behavior {
	inner: PtrCell<yazi_scheduler::Behavior>,
}

impl Deref for Behavior {
	type Target = yazi_scheduler::Behavior;

	fn deref(&self) -> &Self::Target { &self.inner }
}

impl Behavior {
	pub(super) fn make(inner: &yazi_scheduler::Behavior) -> mlua::Result<AnyUserData> {
		let inner = PtrCell::from(inner);

		Lives::scoped_userdata(Self { inner })
	}
}

impl UserData for Behavior {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_method("reset", |_, me, ()| Ok(me.reset()));
	}
}
