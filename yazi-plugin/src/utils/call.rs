use std::collections::HashMap;

use mlua::{ExternalError, Lua, Table, Value};
use yazi_dds::ValueSendable;
use yazi_shared::{emit, event::Cmd, render, Layer};

use super::Utils;

impl Utils {
	fn parse_args(t: Table) -> mlua::Result<(Vec<String>, HashMap<String, String>)> {
		let mut args = vec![];
		let mut named = HashMap::new();
		for result in t.pairs::<Value, Value>() {
			let (k, v) = result?;
			match k {
				Value::Integer(_) => {
					args.push(match v {
						Value::Integer(i) => i.to_string(),
						Value::Number(n) => n.to_string(),
						Value::String(s) => s.to_string_lossy().into_owned(),
						_ => return Err("invalid value in cmd".into_lua_err()),
					});
				}
				Value::String(s) => {
					let v = match v {
						Value::Boolean(b) if b => String::new(),
						Value::Boolean(b) if !b => continue,
						Value::Integer(i) => i.to_string(),
						Value::Number(n) => n.to_string(),
						Value::String(s) => s.to_string_lossy().into_owned(),
						_ => return Err("invalid value in cmd".into_lua_err()),
					};
					named.insert(s.to_str()?.replace('_', "-"), v);
				}
				_ => return Err("invalid key in cmd".into_lua_err()),
			}
		}
		Ok((args, named))
	}

	#[inline]
	fn create_cmd(name: String, table: Table, data: Option<Value>) -> mlua::Result<Cmd> {
		let (args, named) = Self::parse_args(table)?;
		let mut cmd = Cmd { name, args, named, ..Default::default() };

		if let Some(data) = data.and_then(|v| ValueSendable::try_from(v).ok()) {
			cmd = cmd.with_data(data);
		}
		Ok(cmd)
	}

	pub(super) fn call(lua: &Lua, ya: &Table) -> mlua::Result<()> {
		ya.raw_set(
			"render",
			lua.create_function(|_, ()| {
				render!();
				Ok(())
			})?,
		)?;

		ya.raw_set(
			"app_emit",
			lua.create_function(|_, (name, table, data): (String, Table, Option<Value>)| {
				emit!(Call(Self::create_cmd(name, table, data)?, Layer::App));
				Ok(())
			})?,
		)?;

		ya.raw_set(
			"manager_emit",
			lua.create_function(|_, (name, table, data): (String, Table, Option<Value>)| {
				emit!(Call(Self::create_cmd(name, table, data)?, Layer::Manager));
				Ok(())
			})?,
		)?;

		Ok(())
	}
}
