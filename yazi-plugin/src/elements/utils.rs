use mlua::{ExternalError, IntoLua, Lua, ObjectLike, Table, Value};
use tracing::error;
use unicode_width::UnicodeWidthStr;
use yazi_config::LAYOUT;

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

	pub(super) fn redraw(lua: &Lua) -> mlua::Result<Value> {
		let f = lua.create_function(|lua, c: Table| {
			let id: mlua::String = c.get("_id")?;

			let mut layout = LAYOUT.get();
			match id.as_bytes().as_ref() {
				b"current" => layout.current = *c.raw_get::<crate::elements::Rect>("_area")?,
				b"preview" => layout.preview = *c.raw_get::<crate::elements::Rect>("_area")?,
				b"progress" => layout.progress = *c.raw_get::<crate::elements::Rect>("_area")?,
				_ => {}
			}

			LAYOUT.set(layout);
			match c.call_method::<Value>("redraw", ())? {
				Value::Table(tbl) => Ok(tbl),
				Value::UserData(ud) => lua.create_sequence_from([ud]),
				_ => {
					error!(
						"Failed to `redraw()` the `{}` component: expected a table or UserData",
						id.display(),
					);
					lua.create_table()
				}
			}
		})?;

		f.into_lua(lua)
	}
}
