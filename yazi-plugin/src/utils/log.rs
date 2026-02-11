use std::{any::TypeId, fmt::Write};

use mlua::{Function, Lua, MultiValue, Value};
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
			Utils::format_one(&mut s, value)?;
		}
		Ok(s)
	}

	fn format_one(buf: &mut String, value: Value) -> anyhow::Result<()> {
		let Value::UserData(ud) = &value else {
			return Ok(write!(buf, "{value:#?}")?);
		};

		let id = ud.type_id();
		let ptr = ud.to_pointer();
		Ok(match id {
			Some(t) if t == TypeId::of::<yazi_binding::Url>() => {
				write!(buf, "Url({ptr:?}): {:?}", **ud.borrow::<yazi_binding::Url>()?)?
			}
			Some(t) if t == TypeId::of::<yazi_binding::Path>() => {
				write!(buf, "Path({ptr:?}): {:?}", **ud.borrow::<yazi_binding::Path>()?)?
			}
			Some(t) if t == TypeId::of::<yazi_binding::Id>() => {
				write!(buf, "Id({ptr:?}): {}", **ud.borrow::<yazi_binding::Id>()?)?
			}
			_ => write!(buf, "{value:#?}")?,
		})
	}
}
