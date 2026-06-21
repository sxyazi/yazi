use std::{any::TypeId, fmt::Write};

use mlua::{Function, Lua, MultiValue, Value};
use tracing::{debug, error};
use yazi_shared::{id::Id, path::PathBufDyn, url::UrlBuf};

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
			Self::format_one(&mut s, value)?;
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
			Some(t) if t == TypeId::of::<UrlBuf>() => {
				write!(buf, "Url({ptr:?}): {:?}", *ud.borrow::<UrlBuf>()?)?
			}
			Some(t) if t == TypeId::of::<PathBufDyn>() => {
				write!(buf, "Path({ptr:?}): {:?}", *ud.borrow::<PathBufDyn>()?)?
			}
			Some(t) if t == TypeId::of::<Id>() => write!(buf, "Id({ptr:?}): {}", *ud.borrow::<Id>()?)?,
			_ => write!(buf, "{value:#?}")?,
		})
	}
}
