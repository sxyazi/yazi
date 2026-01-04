use std::{borrow::Cow, sync::Arc};

use mlua::{ExternalResult, Function, IntoLua, Lua, MetaMethod, MultiValue, ObjectLike, Table, Value};
use yazi_binding::{runtime, runtime_mut};

use super::LOADER;

pub(super) struct Require;

impl Require {
	pub(super) fn install(lua: &Lua) -> mlua::Result<()> {
		lua.globals().raw_set(
			"require",
			lua.create_async_function(|lua, id: mlua::String| async move {
				let id = id.to_str()?;
				let id = Self::absolute_id(&lua, &id)?;
				LOADER.ensure(&id, |_| ()).await.into_lua_err()?;

				runtime_mut!(lua)?.push(&id);
				let mod_ = LOADER.load(&lua, &id);
				runtime_mut!(lua)?.pop();

				Self::create_mt(&lua, id.into_owned(), mod_?)
			})?,
		)
	}

	fn create_mt(lua: &Lua, id: String, r#mod: Table) -> mlua::Result<Table> {
		let id: Arc<str> = Arc::from(id);
		let mt = lua.create_table_from([
			(
				MetaMethod::Index.name(),
				lua.create_function(move |lua, (ts, key): (Table, mlua::String)| {
					match ts.raw_get::<Table>("__mod")?.raw_get::<Value>(&key)? {
						Value::Function(_) => {
							Self::create_wrapper(lua, id.clone(), &key.to_str()?)?.into_lua(lua)
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
		ts.set_metatable(Some(mt))?;
		Ok(ts)
	}

	fn create_wrapper(lua: &Lua, id: Arc<str>, f: &str) -> mlua::Result<Function> {
		let f: Arc<str> = Arc::from(f);

		lua.create_async_function(move |lua, args: MultiValue| {
			let (id, f) = (id.clone(), f.clone());
			async move {
				let (r#mod, args) = Self::split_mod_and_args(&lua, &id, args)?;
				runtime_mut!(lua)?.push(&id);
				let result = r#mod.call_async_function::<MultiValue>(&f, args).await;
				runtime_mut!(lua)?.pop();
				result
			}
		})
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

	fn absolute_id<'a>(lua: &Lua, id: &'a str) -> mlua::Result<Cow<'a, str>> {
		let Some(stripped) = id.strip_prefix('.') else { return Ok(id.into()) };
		Ok(if let Some(cur) = runtime!(lua)?.current() {
			format!("{}.{stripped}", cur.split('.').next().unwrap_or(cur)).into()
		} else {
			stripped.into()
		})
	}
}
