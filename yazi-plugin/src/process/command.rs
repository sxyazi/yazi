use std::process::Stdio;

use mlua::{AnyUserData, ExternalError, IntoLuaMulti, Lua, Table, UserData, Value};
use tokio::process::{ChildStderr, ChildStdin, ChildStdout};

use super::{Child, output::Output};
use crate::process::Status;

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
			("NULL", NULL),
			("PIPED", PIPED),
			("INHERIT", INHERIT),
		])?;

		command.set_metatable(Some(lua.create_table_from([("__call", new)])?));

		lua.globals().raw_set("Command", command)
	}
}

impl UserData for Command {
	fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
		#[inline]
		fn make_stdio(v: Value) -> mlua::Result<Stdio> {
			match v {
				Value::Integer(n) => {
					return Ok(match n as u8 {
						PIPED => Stdio::piped(),
						INHERIT => Stdio::inherit(),
						_ => Stdio::null(),
					});
				}
				Value::UserData(ud) => {
					if let Ok(stdin) = ud.take::<ChildStdin>() {
						return Ok(stdin.try_into()?);
					} else if let Ok(stdout) = ud.take::<ChildStdout>() {
						return Ok(stdout.try_into()?);
					} else if let Ok(stderr) = ud.take::<ChildStderr>() {
						return Ok(stderr.try_into()?);
					}
				}
				_ => {}
			}

			Err(
				"must be one of Command.NULL, Command.PIPED, Command.INHERIT, or a ChildStdin, ChildStdout, or ChildStderr".into_lua_err(),
			)
		}

		methods.add_function_mut("arg", |_, (ud, arg): (AnyUserData, mlua::String)| {
			ud.borrow_mut::<Self>()?.inner.arg(arg.to_string_lossy());
			Ok(ud)
		});
		methods.add_function_mut("args", |_, (ud, args): (AnyUserData, Vec<mlua::String>)| {
			{
				let mut me = ud.borrow_mut::<Self>()?;
				for arg in args {
					me.inner.arg(arg.to_string_lossy());
				}
			}
			Ok(ud)
		});
		methods.add_function_mut("cwd", |_, (ud, dir): (AnyUserData, mlua::String)| {
			ud.borrow_mut::<Self>()?.inner.current_dir(dir.to_str()?.as_ref());
			Ok(ud)
		});
		methods.add_function_mut(
			"env",
			|_, (ud, key, value): (AnyUserData, mlua::String, mlua::String)| {
				ud.borrow_mut::<Self>()?.inner.env(key.to_string_lossy(), value.to_string_lossy());
				Ok(ud)
			},
		);
		methods.add_function_mut("stdin", |_, (ud, stdio): (AnyUserData, Value)| {
			ud.borrow_mut::<Self>()?.inner.stdin(make_stdio(stdio)?);
			Ok(ud)
		});
		methods.add_function_mut("stdout", |_, (ud, stdio): (AnyUserData, Value)| {
			ud.borrow_mut::<Self>()?.inner.stdout(make_stdio(stdio)?);
			Ok(ud)
		});
		methods.add_function_mut("stderr", |_, (ud, stdio): (AnyUserData, Value)| {
			ud.borrow_mut::<Self>()?.inner.stderr(make_stdio(stdio)?);
			Ok(ud)
		});
		methods.add_method_mut("spawn", |lua, me, ()| match me.inner.spawn() {
			Ok(child) => (Child::new(child), Value::Nil).into_lua_multi(lua),
			Err(e) => (Value::Nil, e.raw_os_error()).into_lua_multi(lua),
		});
		methods.add_async_method_mut("output", |lua, mut me, ()| async move {
			match me.inner.output().await {
				Ok(output) => (Output::new(output), Value::Nil).into_lua_multi(&lua),
				Err(e) => (Value::Nil, e.raw_os_error()).into_lua_multi(&lua),
			}
		});
		methods.add_async_method_mut("status", |lua, mut me, ()| async move {
			match me.inner.status().await {
				Ok(status) => (Status::new(status), Value::Nil).into_lua_multi(&lua),
				Err(e) => (Value::Nil, e.raw_os_error()).into_lua_multi(&lua),
			}
		});
	}
}
