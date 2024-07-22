use std::sync::Arc;

use mlua::{ExternalResult, Function, IntoLua, Lua, MultiValue, Table, Value};

use super::LOADER;
use crate::RtRef;

pub(crate) struct Require;

impl Require {
	pub(crate) fn install(lua: &Lua) -> mlua::Result<()> {
		lua.globals().raw_set(
			"require",
			lua.create_async_function(|lua, id: mlua::String| async move {
				let s = id.to_str()?;
				LOADER.ensure(s).await.into_lua_err()?;

				lua.named_registry_value::<RtRef>("rt")?.push(s);
				let mod_ = LOADER.load(lua, s);
				lua.named_registry_value::<RtRef>("rt")?.pop();

				Self::create_mt(lua, s, mod_?)
			})?,
		)
	}

	fn create_mt<'a>(lua: &'a Lua, id: &str, mod_: Table<'a>) -> mlua::Result<Table<'a>> {
		let ts = lua.create_table_from([("_mod", mod_.into_lua(lua)?)])?;

		let id: Arc<str> = Arc::from(id);
		let mt = lua.create_table_from([(
			"__index",
			lua.create_function(move |lua, (ts, key): (Table, mlua::String)| {
				match ts.raw_get::<_, Table>("_mod")?.raw_get::<_, Value>(&key)? {
					Value::Function(_) => Self::create_wrapper(lua, id.clone(), key.to_str()?)?.into_lua(lua),
					v => Ok(v),
				}
			})?,
		)])?;

		ts.set_metatable(Some(mt));
		Ok(ts)
	}

	fn create_wrapper<'a>(lua: &'a Lua, id: Arc<str>, f: &str) -> mlua::Result<Function<'a>> {
		let f: Arc<str> = Arc::from(f);

		lua.create_async_function(move |lua, (ts, args): (Table, MultiValue)| {
			let (id, f) = (id.clone(), f.clone());
			async move {
				let f: Function = ts.raw_get::<_, Table>("_mod")?.raw_get(&*f)?;
				let args = MultiValue::from_iter([ts.into_lua(lua)?].into_iter().chain(args));

				lua.named_registry_value::<RtRef>("rt")?.push(&id);
				let result = f.call_async::<_, MultiValue>(args).await;
				lua.named_registry_value::<RtRef>("rt")?.pop();

				result
			}
		})
	}
}

// --- Sync
pub(crate) struct RequireSync;

impl RequireSync {
	pub(crate) fn install(lua: &'static Lua) -> mlua::Result<()> {
		lua.globals().raw_set(
			"require",
			lua.create_function(|lua, id: mlua::String| {
				let s = id.to_str()?;
				futures::executor::block_on(LOADER.ensure(s)).into_lua_err()?;

				lua.named_registry_value::<RtRef>("rt")?.push(s);
				let mod_ = LOADER.load(lua, s);
				lua.named_registry_value::<RtRef>("rt")?.pop();

				Self::create_mt(lua, id, mod_?)
			})?,
		)
	}

	fn create_mt(
		lua: &'static Lua,
		id: mlua::String<'static>,
		mod_: Table<'static>,
	) -> mlua::Result<Table<'static>> {
		let ts = lua.create_table_from([("_id", id)])?;

		let mt = lua.create_table_from([(
			"__index",
			lua.create_function(move |lua, (_, key): (Table, mlua::String)| {
				match mod_.raw_get::<_, Value>(key)? {
					Value::Function(f) => Self::create_wrapper(lua, f)?.into_lua(lua),
					v => Ok(v),
				}
			})?,
		)])?;

		ts.set_metatable(Some(mt));
		Ok(ts)
	}

	fn create_wrapper(lua: &'static Lua, f: Function<'static>) -> mlua::Result<Function<'static>> {
		lua.create_function(move |lua, (ts, args): (Table, MultiValue)| {
			let id: mlua::String = ts.raw_get("_id")?;
			let args = MultiValue::from_iter([ts.into_lua(lua)?].into_iter().chain(args));

			lua.named_registry_value::<RtRef>("rt")?.push(id.to_str()?);
			let result = f.call::<_, MultiValue>(args);
			lua.named_registry_value::<RtRef>("rt")?.pop();

			result
		})
	}
}
