use std::process::Stdio;

use mlua::{prelude::LuaUserDataMethods, AnyUserData, IntoLua, Lua, Table, UserData, Value};

use super::{output::Output, Child};

pub struct Command {
	inner: tokio::process::Command,
}

const NULL: u8 = 0;
const PIPED: u8 = 1;
const INHERIT: u8 = 2;

impl Command {
	pub fn install(lua: &Lua) -> mlua::Result<()> {
		let new = lua.create_function(|_, (_, program): (Table, String)| {
			let mut inner = tokio::process::Command::new(program);
			inner.kill_on_drop(true).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());

			Ok(Self { inner })
		})?;

		let command = lua.create_table_from([
			// Stdio
			("NULL", NULL.into_lua(lua)?),
			("PIPED", PIPED.into_lua(lua)?),
			("INHERIT", INHERIT.into_lua(lua)?),
		])?;

		command.set_metatable(Some(lua.create_table_from([("__call", new)])?));

		lua.globals().set("Command", command)
	}
}

impl UserData for Command {
	fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
		methods.add_function("arg", |_, (ud, arg): (AnyUserData, String)| {
			ud.borrow_mut::<Self>()?.inner.arg(arg);
			Ok(ud)
		});
		methods.add_function("args", |_, (ud, args): (AnyUserData, Vec<String>)| {
			ud.borrow_mut::<Self>()?.inner.args(args);
			Ok(ud)
		});
		methods.add_function("env", |_, (ud, key, value): (AnyUserData, String, String)| {
			ud.borrow_mut::<Self>()?.inner.env(key, value);
			Ok(ud)
		});
		methods.add_function("stdin", |_, (ud, stdio): (AnyUserData, u8)| {
			ud.borrow_mut::<Self>()?.inner.stdin(match stdio {
				PIPED => Stdio::piped(),
				INHERIT => Stdio::inherit(),
				_ => Stdio::null(),
			});
			Ok(ud)
		});
		methods.add_function("stdout", |_, (ud, stdio): (AnyUserData, u8)| {
			ud.borrow_mut::<Self>()?.inner.stdout(match stdio {
				PIPED => Stdio::piped(),
				INHERIT => Stdio::inherit(),
				_ => Stdio::null(),
			});
			Ok(ud)
		});
		methods.add_function("stderr", |_, (ud, stdio): (AnyUserData, u8)| {
			ud.borrow_mut::<Self>()?.inner.stderr(match stdio {
				PIPED => Stdio::piped(),
				INHERIT => Stdio::inherit(),
				_ => Stdio::null(),
			});
			Ok(ud)
		});
		methods.add_method_mut("spawn", |lua, me, ()| {
			Ok(match me.inner.spawn() {
				Ok(child) => (Child::new(child).into_lua(lua)?, Value::Nil),
				Err(e) => (Value::Nil, e.raw_os_error().into_lua(lua)?),
			})
		});
		methods.add_async_method_mut("output", |lua, me, ()| async move {
			Ok(match me.inner.output().await {
				Ok(output) => (Output::new(output).into_lua(lua)?, Value::Nil),
				Err(e) => (Value::Nil, e.raw_os_error().into_lua(lua)?),
			})
		});
	}
}
