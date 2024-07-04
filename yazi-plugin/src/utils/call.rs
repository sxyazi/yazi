use std::{collections::HashMap, sync::Arc};

use mlua::{ExternalError, Lua, Table, TableExt, Value};
use tracing::error;
use yazi_config::LAYOUT;
use yazi_dds::Sendable;
use yazi_shared::{emit, event::{Cmd, Data}, render, Layer};

use super::Utils;
use crate::elements::RectRef;

impl Utils {
	fn parse_args(t: Table) -> mlua::Result<HashMap<String, Data>> {
		let mut args = HashMap::with_capacity(t.raw_len());
		for pair in t.pairs::<Value, Value>() {
			let (k, v) = pair?;
			match k {
				Value::Integer(i) if i > 0 => {
					args.insert((i - 1).to_string(), Sendable::value_to_data(v)?);
				}
				Value::String(s) => {
					args.insert(s.to_str()?.replace('_', "-"), Sendable::value_to_data(v)?);
				}
				_ => return Err("invalid key in cmd".into_lua_err()),
			}
		}
		Ok(args)
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
			"render_with",
			lua.create_function(|lua, c: Table| {
				let id: mlua::String = c.get("_id")?;
				let id = id.to_str()?;

				match id {
					"current" => {
						LAYOUT.store(Arc::new(yazi_config::Layout {
							current: *c.raw_get::<_, RectRef>("_area")?,
							..*LAYOUT.load_full()
						}));
					}
					"preview" => {
						LAYOUT.store(Arc::new(yazi_config::Layout {
							preview: *c.raw_get::<_, RectRef>("_area")?,
							..*LAYOUT.load_full()
						}));
					}
					_ => {}
				}

				match c.call_method::<_, Table>("render", ()) {
					Err(e) => {
						error!("Failed to `render()` the `{id}` component:\n{e}");
						lua.create_table()
					}
					ok => ok,
				}
			})?,
		)?;

		ya.raw_set(
			"app_emit",
			lua.create_function(|_, (name, args): (String, Table)| {
				emit!(Call(Cmd { name, args: Self::parse_args(args)? }, Layer::App));
				Ok(())
			})?,
		)?;

		ya.raw_set(
			"manager_emit",
			lua.create_function(|_, (name, args): (String, Table)| {
				emit!(Call(Cmd { name, args: Self::parse_args(args)? }, Layer::Manager));
				Ok(())
			})?,
		)?;

		ya.raw_set(
			"input_emit",
			lua.create_function(|_, (name, args): (String, Table)| {
				emit!(Call(Cmd { name, args: Self::parse_args(args)? }, Layer::Input));
				Ok(())
			})?,
		)?;

		Ok(())
	}
}
