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
		let id: Arc<str> = Arc::from(id);
		let mt = lua.create_table_from([
			(
				"__index",
				lua.create_function(move |lua, (ts, key): (Table, mlua::String)| {
					match ts.raw_get::<_, Table>("__mod")?.raw_get::<_, Value>(&key)? {
						Value::Function(_) => {
							Self::create_wrapper(lua, id.clone(), key.to_str()?, sync)?.into_lua(lua)
						}
						v => Ok(v),
					}
				})?,
			),
			(
				"__newindex",
				lua.create_function(move |_, (ts, key, value): (Table, mlua::String, Value)| {
					ts.raw_get::<_, Table>("__mod")?.raw_set(key, value)
				})?,
			),
		])?;

		let ts = lua.create_table_from([("__mod", mod_)])?;
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
			lua.create_function(move |lua, args: MultiValue| {
				let (mod_, args) = Self::split_mod_and_args(lua, &id, args)?;
				lua.named_registry_value::<RtRef>("rt")?.push(&id);
				let result = mod_.call_function::<_, MultiValue>(&f, args);
				lua.named_registry_value::<RtRef>("rt")?.pop();
				result
			})
		} else {
			lua.create_async_function(move |lua, args: MultiValue| {
				let (id, f) = (id.clone(), f.clone());
				async move {
					let (mod_, args) = Self::split_mod_and_args(lua, &id, args)?;
					lua.named_registry_value::<RtRef>("rt")?.push(&id);
					let result = mod_.call_async_function::<_, MultiValue>(&f, args).await;
					lua.named_registry_value::<RtRef>("rt")?.pop();
					result
				}
			})
		}
	}

	fn split_mod_and_args<'a>(
		lua: &'a Lua,
		id: &str,
		mut args: MultiValue<'a>,
	) -> mlua::Result<(Table<'a>, MultiValue<'a>)> {
		let Some(front) = args.pop_front() else {
			return Ok((LOADER.try_load(lua, id)?, args));
		};
		let Value::Table(tbl) = front else {
			args.push_front(front);
			return Ok((LOADER.try_load(lua, id)?, args));
		};
		Ok(if let Ok(mod_) = tbl.raw_get::<_, Table>("__mod") {
			args.push_front(Value::Table(mod_.clone()));
			(mod_, args)
		} else {
			args.push_front(Value::Table(tbl));
			(LOADER.try_load(lua, id)?, args)
		})
	}
}
