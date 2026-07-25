use std::fmt::Write;

use mlua::{Function, Lua, MultiValue};
use tracing::{debug, error};

use super::Utils;

impl Utils {
	pub(super) fn dbg(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|_, values: MultiValue| Ok(debug!("{}", Self::format_all(values)?)))
	}

	pub(super) fn err(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|_, values: MultiValue| Ok(error!("{}", Self::format_all(values)?)))
	}

	fn format_all(values: MultiValue) -> anyhow::Result<String> {
		let mut s = String::new();
		for value in values {
			if !s.is_empty() {
				s.push(' ');
			}
			write!(s, "{value:#?}")?;
		}
		Ok(s)
	}
}
