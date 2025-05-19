use mlua::{ExternalError, IntoLua, Lua, Value};
use unicode_width::UnicodeWidthStr;

use super::{Line, Span};

pub(super) struct Utils;

impl Utils {
	pub(super) fn width(lua: &Lua) -> mlua::Result<Value> {
		let f = lua.create_function(|_, v: Value| match v {
			Value::String(s) => {
				let (mut acc, b) = (0, s.as_bytes());
				for c in b.utf8_chunks() {
					acc += c.valid().width();
					if !c.invalid().is_empty() {
						acc += 1;
					}
				}
				Ok(acc)
			}
			Value::UserData(ud) => {
				if let Ok(line) = ud.borrow::<Line>() {
					Ok(line.inner.width())
				} else if let Ok(span) = ud.borrow::<Span>() {
					Ok(span.0.width())
				} else {
					Err("expected a string, Line, or Span".into_lua_err())?
				}
			}
			_ => Err("expected a string, Line, or Span".into_lua_err())?,
		})?;

		f.into_lua(lua)
	}
}
