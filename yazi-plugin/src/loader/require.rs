use std::sync::Arc;

use mlua::{ExternalResult, Function, IntoLua, Lua, MetaMethod, MultiValue, ObjectLike, Table, Value};

use super::LOADER;
use crate::RtRefMut;

pub(super) struct Require;

impl Require {
	pub(super) fn install(lua: &Lua) -> mlua::Result<()> {
		lua.globals().raw_set(
			"require",
			lua.create_function(|lua, id: mlua::String| {
				let s = &id.to_str()?;
				futures::executor::block_on(LOADER.ensure(s, |_| ())).into_lua_err()?;

				lua.named_registry_value::<RtRefMut>("ir")?.push(s);
				let mod_ = LOADER.load(lua, s);
				lua.named_registry_value::<RtRefMut>("ir")?.pop();

				Self::create_mt(lua, s, mod_?, true)
			})?,
		)
	}

	pub(super) fn install_isolate(lua: &Lua) -> mlua::Result<()> {
		lua.globals().raw_set(
			"require",
			lua.create_async_function(|lua, id: mlua::String| async move {
				let s = &id.to_str()?;
				LOADER.ensure(s, |_| ()).await.into_lua_err()?;

				lua.named_registry_value::<RtRefMut>("ir")?.push(s);
				let mod_ = LOADER.load(&lua, s);
				lua.named_registry_value::<RtRefMut>("ir")?.pop();

				Self::create_mt(&lua, s, mod_?, false)
			})?,
		)
	}

	fn create_mt(lua: &Lua, id: &str, r#mod: Table, sync: bool) -> mlua::Result<Table> {
		let id: Arc<str> = Arc::from(id);
		let mt = lua.create_table_from([
			(
				MetaMethod::Index.name(),
				lua.create_function(move |lua, (ts, key): (Table, mlua::String)| {
					match ts.raw_get::<Table>("__mod")?.raw_get::<Value>(&key)? {
						Value::Function(_) => {
							Self::create_wrapper(lua, id.clone(), &key.to_str()?, sync)?.into_lua(lua)
						}
						v => Ok(v),
					}
				})?,
			),
			(
				MetaMethod::NewIndex.name(),
				lua.create_function(move |_, (ts, key, value): (Table, mlua::String, Value)| {
					ts.raw_get::<Table>("__mod")?.raw_set(key, value)
				})?,
			),
		])?;

		let ts = lua.create_table_from([("__mod", r#mod)])?;
		ts.set_metatable(Some(mt));
		Ok(ts)
	}

	fn create_wrapper(lua: &Lua, id: Arc<str>, f: &str, sync: bool) -> mlua::Result<Function> {
		let f: Arc<str> = Arc::from(f);

		if sync {
			lua.create_function(move |lua, args: MultiValue| {
				let (r#mod, args) = Self::split_mod_and_args(lua, &id, args)?;
				lua.named_registry_value::<RtRefMut>("ir")?.push(&id);
				let result = r#mod.call_function::<MultiValue>(&f, args);
				lua.named_registry_value::<RtRefMut>("ir")?.pop();
				result
			})
		} else {
			lua.create_async_function(move |lua, args: MultiValue| {
				let (id, f) = (id.clone(), f.clone());
				async move {
					let (r#mod, args) = Self::split_mod_and_args(&lua, &id, args)?;
					lua.named_registry_value::<RtRefMut>("ir")?.push(&id);
					let result = r#mod.call_async_function::<MultiValue>(&f, args).await;
					lua.named_registry_value::<RtRefMut>("ir")?.pop();
					result
				}
			})
		}
	}

	fn split_mod_and_args(
		lua: &Lua,
		id: &str,
		mut args: MultiValue,
	) -> mlua::Result<(Table, MultiValue)> {
		let Some(front) = args.pop_front() else {
			return Ok((LOADER.try_load(lua, id)?, args));
		};
		let Value::Table(tbl) = front else {
			args.push_front(front);
			return Ok((LOADER.try_load(lua, id)?, args));
		};
		Ok(if let Ok(r#mod) = tbl.raw_get::<Table>("__mod") {
			args.push_front(Value::Table(r#mod.clone()));
			(r#mod, args)
		} else {
			args.push_front(Value::Table(tbl));
			(LOADER.try_load(lua, id)?, args)
		})
	}
}
