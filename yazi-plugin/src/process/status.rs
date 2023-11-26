use mlua::UserData;

pub struct Status {
	inner: std::process::ExitStatus,
}

impl Status {
	pub fn new(inner: std::process::ExitStatus) -> Self { Self { inner } }
}

impl UserData for Status {
	fn add_methods<'lua, M: mlua::prelude::LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
		methods.add_method("success", |_, me, ()| Ok(me.inner.success()));
		methods.add_method("code", |_, me, ()| Ok(me.inner.code()));
	}
}
