use std::ops::Deref;

use mlua::{AnyUserData, MetaMethod, MultiValue, ObjectLike, UserData, UserDataFields, UserDataMethods};
use yazi_binding::{Iter, get_metatable};

use super::{Lives, PtrCell};

pub(super) struct Yanked {
	inner: PtrCell<yazi_core::mgr::Yanked>,
	iter:  AnyUserData,
}

impl Deref for Yanked {
	type Target = yazi_core::mgr::Yanked;

	fn deref(&self) -> &Self::Target { &self.inner }
}

impl Yanked {
	pub(super) fn make(inner: &yazi_core::mgr::Yanked) -> mlua::Result<AnyUserData> {
		let inner = PtrCell::from(inner);

		Lives::scoped_userdata(Self {
			inner,
			iter: Lives::scoped_userdata(Iter::new(
				inner.as_static().iter().map(yazi_binding::Url::new),
				Some(inner.len()),
			))?,
		})
	}
}

impl UserData for Yanked {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("is_cut", |_, me| Ok(me.cut));
	}

	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_meta_method(MetaMethod::Len, |_, me, ()| Ok(me.len()));

		methods.add_meta_method(MetaMethod::Pairs, |lua, me, ()| {
			get_metatable(lua, &me.iter)?
				.call_function::<MultiValue>(MetaMethod::Pairs.name(), me.iter.clone())
		});
	}
}
