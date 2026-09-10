use std::ops::Deref;

use futures::TryStreamExt;
use mlua::{AnyUserData, ExternalError, FromLua, FromLuaMulti, Function, IntoLuaMulti, Lua, MetaMethod, MultiValue, Thread, UserData, UserDataMethods, Value};
use yazi_shim::fs::Error;

// --- CoIter
pub struct CoIter {
	handle:  CoHandle,
	started: bool,
}

impl Deref for CoIter {
	type Target = CoHandle;

	fn deref(&self) -> &Self::Target { &self.handle }
}

impl Drop for CoIter {
	fn drop(&mut self) {
		if !self.thread.is_finished() {
			self.thread.reset(self.reset.clone()).ok();
		}
	}
}

impl CoIter {
	pub(crate) async fn next<T>(&mut self, lua: &Lua) -> mlua::Result<Option<T>>
	where
		T: FromLuaMulti,
	{
		let mut values = if self.started {
			self.resume(true).await?
		} else {
			self.started = true;
			self.resume(()).await?
		};

		if !values.front().is_none_or(Value::is_nil) {
			return T::from_lua_multi(values, lua).map(Some);
		}

		let _ = values.pop_front();
		if let Some(v) = values.pop_front().filter(|v| !v.is_nil()) {
			return Err(Error::from_lua(v, lua)?.into_lua_err());
		}

		Ok(None)
	}
}

impl FromLua for CoIter {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
		Ok(Self { handle: AnyUserData::from_lua(value, lua)?.take()?, started: false })
	}
}

// --- CoHandle
pub struct CoHandle {
	thread: Thread,
	reset:  Function,
}

impl CoHandle {
	pub fn create(lua: &Lua, f: Function) -> mlua::Result<AnyUserData> {
		lua.create_userdata(Self { thread: lua.create_thread(f.clone())?, reset: f })
	}

	async fn resume(&self, args: impl IntoLuaMulti) -> mlua::Result<MultiValue> {
		let values: MultiValue = self.thread.resume(args)?;

		if let Some(Value::LightUserData(ud)) = values.front()
			&& *ud == Lua::poll_pending()
		{
			let mut thread = self.thread.clone().into_async(())?;
			return Ok(thread.try_next().await?.unwrap_or_default());
		}

		Ok(values)
	}
}

impl UserData for CoHandle {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_async_meta_method(MetaMethod::Call, |_, me, args: MultiValue| async move {
			me.resume(args).await
		});
	}
}
