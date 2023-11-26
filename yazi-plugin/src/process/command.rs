use std::process::Stdio;

use mlua::{prelude::LuaUserDataMethods, AnyUserData, IntoLua, Lua, UserData};

use super::{output::Output, Child};

pub struct Command {
	inner: tokio::process::Command,
}

const NULL: u8 = 0;
const PIPED: u8 = 1;
const INHERIT: u8 = 2;

impl Command {
	pub fn install(lua: &Lua) -> mlua::Result<()> {
		let new = lua.create_function(|_, program: String| {
			let mut inner = tokio::process::Command::new(program);
			inner.kill_on_drop(true).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());

			Ok(Self { inner })
		})?;

		lua.globals().set(
			"Command",
			lua.create_table_from([
				("new", new.into_lua(lua)?),
				// Stdio
				("NULL", NULL.into_lua(lua)?),
				("PIPED", PIPED.into_lua(lua)?),
				("INHERIT", INHERIT.into_lua(lua)?),
			])?,
		)
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
		methods.add_method_mut("spawn", |_, me, ()| Ok(Child::new(me.inner.spawn()?)));
		methods.add_async_method_mut("output", |_, me, ()| async move {
			Ok(Output::new(me.inner.output().await?))
		});
	}
}
