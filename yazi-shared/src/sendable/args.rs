use std::borrow::Cow;

use hashbrown::HashMap;
use mlua::{ExternalError, Lua, Table, Value};

use super::Sendable;
use crate::{data::{Data, DataKey}, replace_cow};

impl Sendable {
	pub fn table_to_args(lua: &Lua, t: Table) -> mlua::Result<HashMap<DataKey, Data>> {
		let mut args = HashMap::with_capacity(t.raw_len());
		for pair in t.pairs::<Value, Value>() {
			let (k, v) = pair?;
			match k {
				Value::Integer(i) if i > 0 => {
					args.insert(DataKey::Integer(i - 1), Self::value_to_data(lua, v)?);
				}
				Value::String(s) => {
					args.insert(
						DataKey::String(Cow::Owned(s.to_str()?.replace('_', "-"))),
						Self::value_to_data(lua, v)?,
					);
				}
				_ => return Err("invalid key in Action".into_lua_err()),
			}
		}
		Ok(args)
	}

	pub fn args_to_table(lua: &Lua, args: HashMap<DataKey, Data>) -> mlua::Result<Table> {
		let seq_len = args.keys().filter(|&k| k.is_integer()).count();
		let tbl = lua.create_table_with_capacity(seq_len, args.len() - seq_len)?;
		for (k, v) in args {
			match k {
				DataKey::Integer(i) => tbl.raw_set(i + 1, Self::data_to_value(lua, v)?),
				DataKey::String(s) => tbl.raw_set(replace_cow(s, "-", "_"), Self::data_to_value(lua, v)?),
				_ => Err("invalid key in Data".into_lua_err()),
			}?;
		}
		Ok(tbl)
	}

	pub fn args_to_table_ref(lua: &Lua, args: &HashMap<DataKey, Data>) -> mlua::Result<Table> {
		let seq_len = args.keys().filter(|&k| k.is_integer()).count();
		let tbl = lua.create_table_with_capacity(seq_len, args.len() - seq_len)?;
		for (k, v) in args {
			match k {
				DataKey::Integer(i) => tbl.raw_set(i + 1, Self::data_to_value_ref(lua, v)?),
				DataKey::String(s) => {
					tbl.raw_set(replace_cow(&**s, "-", "_"), Self::data_to_value_ref(lua, v)?)
				}
				_ => Err("invalid key in Data".into_lua_err()),
			}?;
		}
		Ok(tbl)
	}
}
