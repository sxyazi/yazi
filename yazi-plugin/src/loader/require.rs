use mlua::{ExternalResult, Function, IntoLua, Lua, MetaMethod, Table, TableExt, UserData, Value, Variadic};

use crate::RtRef;

pub(super) struct Require;

impl Require {
	pub(super) fn install(lua: &'static Lua) -> mlua::Result<()> {
		let globals = lua.globals();

		let require = globals.raw_get::<_, Function>("require")?;
		globals.raw_set(
			"require",
			lua.create_function(move |lua, name: mlua::String| {
				lua.named_registry_value::<RtRef>("rt")?.swap(name.to_str()?);
				let module: Table = require.call(&name)?;
				lua.named_registry_value::<RtRef>("rt")?.reset();

				module.raw_set("_name", &name)?;
				Self::create_mt(lua, name, module)
			})?,
		)?;

		Ok(())
	}

	fn create_mt(
		lua: &'static Lua,
		name: mlua::String<'static>,
		module: Table<'static>,
	) -> mlua::Result<Table<'static>> {
		let ts =
			lua.create_table_from([("name", name.into_lua(lua)?), ("module", module.into_lua(lua)?)])?;

		let mt = lua.create_table()?;
		mt.raw_set(
			"__index",
			lua.create_function(|_, (ts, key): (Table, mlua::String)| {
				if key.to_str()? == "setup" {
					Ok(RequireSetup { name: ts.raw_get("name")?, module: ts.raw_get("module")? })
				} else {
					Err("Only `require():setup()` and `require().setup()` are supported").into_lua_err()
				}
			})?,
		)?;

		ts.set_metatable(Some(mt));
		Ok(ts)
	}
}

pub(super) struct RequireSetup {
	name:   mlua::String<'static>,
	module: Table<'static>,
}

impl UserData for RequireSetup {
	fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
		methods.add_meta_method(MetaMethod::Call, |lua, me, args: Variadic<Value>| {
			lua.named_registry_value::<RtRef>("rt")?.swap(me.name.to_str()?);
			let result = me.module.call_function::<_, Variadic<Value>>("setup", args);
			lua.named_registry_value::<RtRef>("rt")?.reset();
			result
		});
	}
}
