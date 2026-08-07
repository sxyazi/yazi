use std::{borrow::Cow, sync::Arc};

use mlua::{Function, IntoLua, Lua, LuaString, MetaMethod, MultiValue, ObjectLike, Table, Value};
use yazi_binding::{runtime, runtime_mut};

use super::LOADER;

pub(super) struct Require;

impl Require {
	pub(super) fn install(lua: &Lua) -> mlua::Result<()> {
		lua.globals().raw_set(
			"require",
			lua.create_async_function(|lua, name: LuaString| async move {
				let name = name.to_str()?;
				let name = Self::absolute_name(&lua, &name)?;
				LOADER.ensure(&name, |_| ()).await?;

				runtime_mut!(lua)?.enter_nested(&name);
				let mod_ = LOADER.load(&lua, &name).await;
				runtime_mut!(lua)?.leave()?;

				Self::create_mt(&lua, name.into_owned(), mod_?)
			})?,
		)
	}

	fn create_mt(lua: &Lua, name: String, r#mod: Table) -> mlua::Result<Table> {
		let name: Arc<str> = Arc::from(name);
		let mt = lua.create_table_from([
			(
				MetaMethod::Index.name(),
				lua.create_function(move |lua, (ts, key): (Table, LuaString)| {
					match ts.raw_get::<Table>("__mod")?.raw_get::<Value>(&key)? {
						Value::Function(_) => {
							Self::create_wrapper(lua, name.clone(), &key.to_str()?)?.into_lua(lua)
						}
						v => Ok(v),
					}
				})?,
			),
			(
				MetaMethod::NewIndex.name(),
				lua.create_function(move |_, (ts, key, value): (Table, LuaString, Value)| {
					ts.raw_get::<Table>("__mod")?.raw_set(key, value)
				})?,
			),
		])?;

		let ts = lua.create_table_from([("__mod", r#mod)])?;
		ts.set_metatable(Some(mt))?;
		Ok(ts)
	}

	fn create_wrapper(lua: &Lua, name: Arc<str>, f: &str) -> mlua::Result<Function> {
		let f: Arc<str> = Arc::from(f);

		lua.create_async_function(move |lua, args: MultiValue| {
			let (name, f) = (name.clone(), f.clone());
			async move {
				let (r#mod, args) = Self::split_mod_and_args(&lua, &name, args)?;
				runtime_mut!(lua)?.enter_nested(&name);
				let result = r#mod.call_async_function::<MultiValue>(&f, args).await;
				runtime_mut!(lua)?.leave()?;
				result
			}
		})
	}

	fn split_mod_and_args(
		lua: &Lua,
		name: &str,
		mut args: MultiValue,
	) -> mlua::Result<(Table, MultiValue)> {
		let Some(front) = args.pop_front() else {
			return Ok((LOADER.try_load(lua, name)?, args));
		};
		let Value::Table(tbl) = front else {
			args.push_front(front);
			return Ok((LOADER.try_load(lua, name)?, args));
		};
		Ok(if let Ok(r#mod) = tbl.raw_get::<Table>("__mod") {
			args.push_front(Value::Table(r#mod.clone()));
			(r#mod, args)
		} else {
			args.push_front(Value::Table(tbl));
			(LOADER.try_load(lua, name)?, args)
		})
	}

	fn absolute_name<'a>(lua: &Lua, name: &'a str) -> mlua::Result<Cow<'a, str>> {
		Ok(match name.strip_prefix('.') {
			None => name.strip_suffix(".main").unwrap_or(name).into(),
			Some("main") => runtime!(lua)?.module()?.to_owned().into(),
			Some(rel) => format!("{}.{rel}", runtime!(lua)?.module()?).into(),
		})
	}
}
