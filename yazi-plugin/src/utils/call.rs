use std::collections::BTreeMap;

use mlua::{ExternalError, Lua, Table, Value};
use yazi_shared::{emit, event::Exec, render, Layer};

use super::Utils;
use crate::ValueSendable;

impl Utils {
	fn parse_args(t: Table) -> mlua::Result<(Vec<String>, BTreeMap<String, String>)> {
		let mut args = vec![];
		let mut named = BTreeMap::new();
		for result in t.pairs::<Value, Value>() {
			let (k, Value::String(v)) = result? else {
				return Err("invalid value in exec".into_lua_err());
			};

			match k {
				Value::Integer(_) => {
					args.push(v.to_str()?.to_owned());
				}
				Value::String(s) => {
					named.insert(s.to_str()?.replace('_', "-"), v.to_str()?.to_owned());
				}
				_ => return Err("invalid key in exec".into_lua_err()),
			}
		}
		Ok((args, named))
	}

	pub(super) fn call(lua: &Lua, ya: &Table) -> mlua::Result<()> {
		ya.set(
			"render",
			lua.create_function(|_, ()| {
				render!();
				Ok(())
			})?,
		)?;

		ya.set(
			"manager_emit",
			lua.create_async_function(
				|_, (cmd, table, data): (String, Table, Option<Value>)| async move {
					let (args, named) = Self::parse_args(table)?;
					let mut exec = Exec { cmd, args, named, ..Default::default() };

					if let Some(data) = data.and_then(|v| ValueSendable::try_from(v).ok()) {
						exec = exec.with_data(data);
					}

					emit!(Call(exec.vec(), Layer::Manager));
					Ok(())
				},
			)?,
		)?;

		Ok(())
	}
}
