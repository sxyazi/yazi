use std::{ffi::OsString, process::Stdio};

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

		let command =
			lua.create_table_from([("NULL", NULL), ("PIPED", PIPED), ("INHERIT", INHERIT)])?;

		command.set_metatable(Some(lua.create_table_from([("__call", new)])?));
		lua.globals().raw_set("Command", command)
	}
}

impl UserData for Command {
	fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
		#[inline]
		fn make_stdio(v: Value) -> mlua::Result<Stdio> {
			match v {
                Value::Integer(n) => Ok(match n as u8 {
                    PIPED => Stdio::piped(),
                    INHERIT => Stdio::inherit(),
                    _ => Stdio::null(),
                }),
                Value::UserData(ud) => {
                    if let Ok(stdin) = ud.take::<ChildStdin>() {
                        Ok(stdin.try_into()?)
                    } else if let Ok(stdout) = ud.take::<ChildStdout>() {
                        Ok(stdout.try_into()?)
                    } else if let Ok(stderr) = ud.take::<ChildStderr>() {
                        Ok(stderr.try_into()?)
                    } else {
                        Err("Invalid userdata type".into_lua_err())
                    }
                }
                _ => Err(
                    "must be one of Command.NULL, Command.PIPED, Command.INHERIT, or a ChildStdin, ChildStdout, or ChildStderr"
                        .into_lua_err(),
                ),
            }
		}

		methods.add_function_mut("arg", |_, (ud, arg): (AnyUserData, mlua::String)| {
			let arg = OsString::from(arg.to_str()?);
			ud.borrow_mut::<Self>()?.inner.arg(arg);
			Ok(ud)
		});

		methods.add_function_mut("args", |_, (ud, args): (AnyUserData, Vec<mlua::String>)| {
			let args: Vec<OsString> =
				args.iter().map(|arg| OsString::from(arg.to_str().unwrap())).collect();
			ud.borrow_mut::<Self>()?.inner.args(&args);
			Ok(ud)
		});

		methods.add_function_mut("cwd", |_, (ud, dir): (AnyUserData, mlua::String)| {
			let dir = OsString::from(dir.to_str()?);
			ud.borrow_mut::<Self>()?.inner.current_dir(dir);
			Ok(ud)
		});

		methods.add_function_mut(
			"env",
			|_, (ud, key, value): (AnyUserData, mlua::String, mlua::String)| {
				let key = OsString::from(key.to_str()?);
				let value = OsString::from(value.to_str()?);
				ud.borrow_mut::<Self>()?.inner.env(key, value);
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

		methods.add_async_method_mut("output", |lua, me, ()| async move {
			match me.inner.output().await {
				Ok(output) => (Output::new(output), Value::Nil).into_lua_multi(lua),
				Err(e) => (Value::Nil, e.raw_os_error()).into_lua_multi(lua),
			}
		});

		methods.add_async_method_mut("status", |lua, me, ()| async move {
			match me.inner.status().await {
				Ok(status) => (Status::new(status), Value::Nil).into_lua_multi(lua),
				Err(e) => (Value::Nil, e.raw_os_error()).into_lua_multi(lua),
			}
		});
	}
}
