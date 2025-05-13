use mlua::{Function, Lua, Table};
use twox_hash::XxHash3_128;
use unicode_width::UnicodeWidthChar;

use super::Utils;
use crate::CLIPBOARD;

impl Utils {
	pub(super) fn hash(lua: &Lua) -> mlua::Result<Function> {
		lua.create_async_function(move |_, s: mlua::String| async move {
			Ok(format!("{:x}", XxHash3_128::oneshot(&s.as_bytes())))
		})
	}

	pub(super) fn quote(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|lua, (s, unix): (mlua::String, Option<bool>)| {
			let s = s.to_str()?;
			let s = match unix {
				Some(true) => yazi_shared::shell::escape_unix(s.as_ref()),
				Some(false) => yazi_shared::shell::escape_windows(s.as_ref()),
				None => yazi_shared::shell::escape_native(s.as_ref()),
			};
			lua.create_string(s.as_ref())
		})
	}

	pub(super) fn truncate(lua: &Lua) -> mlua::Result<Function> {
		fn idx_and_width(it: impl Iterator<Item = (usize, char)>, max: usize) -> (usize, usize) {
			let mut width = 0;
			let idx = it
				.take_while(|(_, c)| {
					width += c.width().unwrap_or(0);
					width <= max
				})
				.map(|(i, _)| i)
				.last()
				.unwrap();
			(idx, width)
		}

		lua.create_function(|lua, (s, t): (mlua::String, Table)| {
			let b = s.as_bytes();
			if b.is_empty() {
				return Ok(s);
			}

			let max = t.raw_get("max")?;
			if b.len() <= max {
				return Ok(s);
			} else if max < 1 {
				return lua.create_string("");
			}

			let lossy = String::from_utf8_lossy(&b);
			let rtl = t.raw_get("rtl").unwrap_or(false);
			let (idx, width) = if rtl {
				idx_and_width(lossy.char_indices().rev(), max)
			} else {
				idx_and_width(lossy.char_indices(), max)
			};

			if width <= max {
				return Ok(s);
			} else if rtl && idx == 0 {
				return Ok(s);
			} else if !rtl && lossy[idx..].chars().nth(1).is_none() {
				return Ok(s);
			}

			let result: Vec<_> = if rtl {
				let i = lossy[idx..].char_indices().nth(1).map(|(i, _)| idx + i).unwrap_or(lossy.len());
				"…".bytes().chain(lossy[i..].bytes()).collect()
			} else {
				lossy[..idx].bytes().chain("…".bytes()).collect()
			};
			lua.create_string(result)
		})
	}

	pub(super) fn clipboard(lua: &Lua) -> mlua::Result<Function> {
		lua.create_async_function(|lua, text: Option<String>| async move {
			if let Some(text) = text {
				CLIPBOARD.set(text).await;
				Ok(None)
			} else {
				Some(lua.create_string(CLIPBOARD.get().await.as_encoded_bytes())).transpose()
			}
		})
	}
}
