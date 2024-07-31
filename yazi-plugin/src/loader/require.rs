use std::sync::Arc;

use mlua::{ExternalResult, Function, IntoLua, Lua, MultiValue, Table, TableExt, Value};

use super::LOADER;
use crate::RtRef;

pub(super) struct Require;

impl Require {
	pub(super) fn install(lua: &Lua) -> mlua::Result<()> {
		lua.globals().raw_set(
			"require",
			lua.create_function(|lua, id: mlua::String| {
				let s = id.to_str()?;
				futures::executor::block_on(LOADER.ensure(s)).into_lua_err()?;

				lua.named_registry_value::<RtRef>("rt")?.push(s);
				let mod_ = LOADER.load(lua, s);
				lua.named_registry_value::<RtRef>("rt")?.pop();

				Self::create_mt(lua, s, mod_?, true)
			})?,
		)
	}

	pub(super) fn install_isolate(lua: &Lua) -> mlua::Result<()> {
		lua.globals().raw_set(
			"require",
			lua.create_async_function(|lua, id: mlua::String| async move {
				let s = id.to_str()?;
				LOADER.ensure(s).await.into_lua_err()?;

				lua.named_registry_value::<RtRef>("rt")?.push(s);
				let mod_ = LOADER.load(lua, s);
				lua.named_registry_value::<RtRef>("rt")?.pop();

				Self::create_mt(lua, s, mod_?, false)
			})?,
		)
	}

	fn create_mt<'a>(lua: &'a Lua, id: &str, mod_: Table<'a>, sync: bool) -> mlua::Result<Table<'a>> {
		let ts = lua.create_table_from([("_mod", mod_.into_lua(lua)?)])?;

		let id: Arc<str> = Arc::from(id);
		let mt = lua.create_table_from([(
			"__index",
			lua.create_function(move |lua, (ts, key): (Table, mlua::String)| {
				match ts.raw_get::<_, Table>("_mod")?.raw_get::<_, Value>(&key)? {
					Value::Function(_) => {
						Self::create_wrapper(lua, id.clone(), key.to_str()?, sync)?.into_lua(lua)
					}
					v => Ok(v),
				}
			})?,
		)])?;

		ts.set_metatable(Some(mt));
		Ok(ts)
	}

	fn create_wrapper<'a>(
		lua: &'a Lua,
		id: Arc<str>,
		f: &str,
		sync: bool,
	) -> mlua::Result<Function<'a>> {
		let f: Arc<str> = Arc::from(f);

		if sync {
			lua.create_function(move |lua, (ts, args): (Table, MultiValue)| {
				let mod_: Table = ts.raw_get::<_, Table>("_mod")?;
				lua.named_registry_value::<RtRef>("rt")?.push(&id);
				let result = mod_.call_method::<_, MultiValue>(&f, args);
				lua.named_registry_value::<RtRef>("rt")?.pop();
				result
			})
		} else {
			lua.create_async_function(move |lua, (ts, args): (Table, MultiValue)| {
				let (id, f) = (id.clone(), f.clone());
				async move {
					let mod_: Table = ts.raw_get::<_, Table>("_mod")?;
					lua.named_registry_value::<RtRef>("rt")?.push(&id);
					let result = mod_.call_async_method::<_, MultiValue>(&f, args).await;
					lua.named_registry_value::<RtRef>("rt")?.pop();
					result
				}
			})
		}
	}
}
