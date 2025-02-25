use std::ops::Deref;

use mlua::{AnyUserData, IntoLua, MetaMethod, UserData, Value};

use super::Lives;

pub(super) struct Ctx {
	inner: *const crate::Ctx,
}

impl Deref for Ctx {
	type Target = crate::Ctx;

	fn deref(&self) -> &Self::Target { unsafe { &*self.inner } }
}

impl Ctx {
	#[inline]
	pub(super) fn make(inner: &crate::Ctx) -> mlua::Result<AnyUserData> {
		Lives::scoped_userdata(Self { inner })
	}
}

impl UserData for Ctx {
	fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
		methods.add_meta_method(MetaMethod::Index, |lua, me, key: mlua::String| {
			match key.as_bytes().as_ref() {
				b"active" => super::Tab::make(me.active())?,
				b"tabs" => super::Tabs::make(&me.mgr.tabs)?,
				b"tasks" => super::Tasks::make(&me.tasks)?,
				b"yanked" => super::Yanked::make(&me.mgr.yanked)?,
				b"layer" => return yazi_plugin::bindings::Layer::from(me.layer()).into_lua(lua),
				_ => return Ok(Value::Nil),
			}
			.into_lua(lua)
		});
	}
}
