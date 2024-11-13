use std::collections::HashMap;

use mlua::{ExternalError, Function, Lua, ObjectLike, Table, Value};
use tracing::error;
use yazi_config::LAYOUT;
use yazi_dds::Sendable;
use yazi_macro::{emit, render};
use yazi_shared::{Layer, event::{Cmd, Data}};

use super::Utils;

impl Utils {
	pub(super) fn render(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|_, ()| {
			render!();
			Ok(())
		})
	}

	pub(super) fn redraw_with(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|lua, c: Table| {
			let id: mlua::String = c.get("_id")?;

			let mut layout = LAYOUT.get();
			match id.as_bytes().as_ref() {
				b"current" => layout.current = *c.raw_get::<crate::elements::Rect>("_area")?,
				b"preview" => layout.preview = *c.raw_get::<crate::elements::Rect>("_area")?,
				b"progress" => layout.progress = *c.raw_get::<crate::elements::Rect>("_area")?,
				_ => {}
			}

			LAYOUT.set(layout);
			match c.call_method::<Table>("redraw", ()) {
				Err(e) => {
					error!("Failed to `redraw()` the `{}` component:\n{e}", id.display());
					lua.create_table()
				}
				ok => ok,
			}
		})
	}

	pub(super) fn app_emit(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|_, (name, args): (String, Table)| {
			emit!(Call(Cmd { name, args: Self::parse_args(args)? }, Layer::App));
			Ok(())
		})
	}

	pub(super) fn manager_emit(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|_, (name, args): (String, Table)| {
			emit!(Call(Cmd { name, args: Self::parse_args(args)? }, Layer::Manager));
			Ok(())
		})
	}

	pub(super) fn input_emit(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|_, (name, args): (String, Table)| {
			emit!(Call(Cmd { name, args: Self::parse_args(args)? }, Layer::Input));
			Ok(())
		})
	}

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
}
