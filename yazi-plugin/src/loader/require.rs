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
				let mod_: Table = require.call(&name)?;
				lua.named_registry_value::<RtRef>("rt")?.reset();

				mod_.raw_set("_name", &name)?;
				Self::create_mt(lua, name, mod_)
			})?,
		)?;

		Ok(())
	}

	fn create_mt(
		lua: &'static Lua,
		name: mlua::String<'static>,
		mod_: Table<'static>,
	) -> mlua::Result<Table<'static>> {
		let ts =
			lua.create_table_from([("name", name.into_lua(lua)?), ("mod", mod_.into_lua(lua)?)])?;

		let mt = lua.create_table_from([(
			"__index",
			lua.create_function(|_, (_, key): (Table, mlua::String)| {
				if key.to_str()? == "setup" {
					Ok(RequireSetup)
				} else {
					Err("Only `require():setup()` is supported").into_lua_err()
				}
			})?,
		)])?;

		ts.set_metatable(Some(mt));
		Ok(ts)
	}
}

pub(super) struct RequireSetup;

impl UserData for RequireSetup {
	fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
		methods.add_meta_method(MetaMethod::Call, |lua, _, (ts, args): (Table, Variadic<Value>)| {
			let (name, mod_): (mlua::String, Table) = (ts.raw_get("name")?, ts.raw_get("mod")?);
			lua.named_registry_value::<RtRef>("rt")?.swap(name.to_str()?);
			let result = mod_.call_method::<_, Variadic<Value>>("setup", args);
			lua.named_registry_value::<RtRef>("rt")?.reset();
			result
		});
	}
}
