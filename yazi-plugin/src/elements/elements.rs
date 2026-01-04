use std::borrow::Cow;

use mlua::{AnyUserData, ExternalError, IntoLua, Lua, ObjectLike, Table, Value};
use tracing::error;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};
use yazi_binding::{Composer, ComposerGet, ComposerSet, Permit, PermitRef, elements::{Line, Rect, Span}, runtime};
use yazi_config::LAYOUT;
use yazi_proxy::{AppProxy, HIDER};
use yazi_shared::replace_to_printable;

pub fn compose() -> Composer<ComposerGet, ComposerSet> {
	fn get(lua: &Lua, key: &[u8]) -> mlua::Result<Value> {
		match key {
			b"area" => area(lua)?,
			b"hide" => hide(lua)?,
			b"printable" => printable(lua)?,
			b"redraw" => redraw(lua)?,
			b"render" => render(lua)?,
			b"truncate" => truncate(lua)?,
			b"width" => width(lua)?,
			_ => return Ok(Value::Nil),
		}
		.into_lua(lua)
	}

	fn set(_: &Lua, _: &[u8], value: Value) -> mlua::Result<Value> { Ok(value) }

	yazi_binding::elements::compose(get, set)
}

pub(super) fn area(lua: &Lua) -> mlua::Result<Value> {
	let f = lua.create_function(|_, s: mlua::String| {
		let layout = LAYOUT.get();
		Ok(match &*s.as_bytes() {
			b"current" => Rect(layout.current),
			b"preview" => Rect(layout.preview),
			b"progress" => Rect(layout.progress),
			_ => Err(format!("unknown area: {}", s.display()).into_lua_err())?,
		})
	})?;

	f.into_lua(lua)
}

pub(super) fn hide(lua: &Lua) -> mlua::Result<Value> {
	let f = lua.create_async_function(|lua, ()| async move {
		if runtime!(lua)?.initing {
			return Err("Cannot call `ui.hide()` during app initialization".into_lua_err());
		}

		if lua.named_registry_value::<PermitRef>("HIDE_PERMIT").is_ok_and(|h| h.is_some()) {
			return Err("Cannot hide while already hidden".into_lua_err());
		}

		let permit = HIDER.acquire().await.unwrap();
		AppProxy::stop().await;

		lua.set_named_registry_value("HIDE_PERMIT", Permit::new(permit, AppProxy::resume()))?;
		lua.named_registry_value::<AnyUserData>("HIDE_PERMIT")
	})?;

	f.into_lua(lua)
}

pub(super) fn printable(lua: &Lua) -> mlua::Result<Value> {
	let f = lua.create_function(|lua, s: mlua::String| {
		Ok(match replace_to_printable(&s.as_bytes(), false, 1, true) {
			Cow::Borrowed(_) => s,
			Cow::Owned(new) => lua.create_string(&new)?,
		})
	})?;

	f.into_lua(lua)
}

pub(super) fn redraw(lua: &Lua) -> mlua::Result<Value> {
	let f = lua.create_function(|lua, c: Table| {
		let id: mlua::String = c.get("_id")?;

		let mut layout = LAYOUT.get();
		match &*id.as_bytes() {
			b"current" => layout.current = *c.raw_get::<Rect>("_area")?,
			b"preview" => layout.preview = *c.raw_get::<Rect>("_area")?,
			b"progress" => layout.progress = *c.raw_get::<Rect>("_area")?,
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

pub(super) fn render(lua: &Lua) -> mlua::Result<Value> {
	let f = lua.create_function(|_, ()| {
		yazi_macro::render!();
		Ok(())
	})?;

	f.into_lua(lua)
}

pub(super) fn truncate(lua: &Lua) -> mlua::Result<Value> {
	fn traverse(it: impl Iterator<Item = (usize, char)>, max: usize) -> (Option<usize>, usize, bool) {
		let (mut adv, mut last) = (0, 0);
		let idx = it
			.take_while(|&(_, c)| {
				(last, adv) = (adv, adv + c.width().unwrap_or(0));
				adv <= max
			})
			.map(|(i, _)| i)
			.last();
		(idx, last, adv > max)
	}

	let f = lua.create_function(|lua, (s, t): (mlua::String, Table)| {
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
		let (idx, width, remain) = if rtl {
			traverse(lossy.char_indices().rev(), max)
		} else {
			traverse(lossy.char_indices(), max)
		};

		let Some(idx) = idx else { return lua.create_string("…") };
		if !remain {
			return Ok(s);
		}

		let result: Vec<_> = match (rtl, width == max) {
			(false, false) => {
				let len = lossy[idx..].chars().next().map_or(0, |c| c.len_utf8());
				lossy[..idx + len].bytes().chain("…".bytes()).collect()
			}
			(false, true) => lossy[..idx].bytes().chain("…".bytes()).collect(),
			(true, false) => "…".bytes().chain(lossy[idx..].bytes()).collect(),
			(true, true) => {
				let len = lossy[idx..].chars().next().map_or(0, |c| c.len_utf8());
				"…".bytes().chain(lossy[idx + len..].bytes()).collect()
			}
		};
		lua.create_string(result)
	})?;

	f.into_lua(lua)
}

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
				Ok(line.width())
			} else if let Ok(span) = ud.borrow::<Span>() {
				Ok(span.width())
			} else {
				Err("expected a string, Line, or Span".into_lua_err())?
			}
		}
		_ => Err("expected a string, Line, or Span".into_lua_err())?,
	})?;

	f.into_lua(lua)
}

#[cfg(test)]
mod tests {
	use mlua::{Lua, chunk};

	fn truncate(s: &str, max: usize, rtl: bool) -> String {
		let lua = Lua::new();
		let f = super::truncate(&lua).unwrap();

		lua
			.load(chunk! {
				return $f($s, { max = $max, rtl = $rtl })
			})
			.call(())
			.unwrap()
	}

	#[test]
	fn test_truncate() {
		assert_eq!(truncate("你好，world", 0, false), "");
		assert_eq!(truncate("你好，world", 1, false), "…");
		assert_eq!(truncate("你好，world", 2, false), "…");

		assert_eq!(truncate("你好，世界", 3, false), "你…");
		assert_eq!(truncate("你好，世界", 4, false), "你…");
		assert_eq!(truncate("你好，世界", 5, false), "你好…");

		assert_eq!(truncate("Hello, world", 5, false), "Hell…");
		assert_eq!(truncate("Ni好，世界", 3, false), "Ni…");
	}

	#[test]
	fn test_truncate_rtl() {
		assert_eq!(truncate("world，你好", 0, true), "");
		assert_eq!(truncate("world，你好", 1, true), "…");
		assert_eq!(truncate("world，你好", 2, true), "…");

		assert_eq!(truncate("你好，世界", 3, true), "…界");
		assert_eq!(truncate("你好，世界", 4, true), "…界");
		assert_eq!(truncate("你好，世界", 5, true), "…世界");

		assert_eq!(truncate("Hello, world", 5, true), "…orld");
		assert_eq!(truncate("你好，Shi界", 3, true), "…界");
	}

	#[test]
	fn test_truncate_oboe() {
		assert_eq!(truncate("Hello, world", 11, false), "Hello, wor…");
		assert_eq!(truncate("你好，世界", 9, false), "你好，世…");
		assert_eq!(truncate("你好，世Jie", 9, false), "你好，世…");

		assert_eq!(truncate("Hello, world", 11, true), "…llo, world");
		assert_eq!(truncate("你好，世界", 9, true), "…好，世界");
		assert_eq!(truncate("Ni好，世界", 9, true), "…好，世界");
	}

	#[test]
	fn test_truncate_exact() {
		assert_eq!(truncate("Hello, world", 12, false), "Hello, world");
		assert_eq!(truncate("你好，世界", 10, false), "你好，世界");

		assert_eq!(truncate("Hello, world", 12, true), "Hello, world");
		assert_eq!(truncate("你好，世界", 10, true), "你好，世界");
	}

	#[test]
	fn test_truncate_overflow() {
		assert_eq!(truncate("Hello, world", 13, false), "Hello, world");
		assert_eq!(truncate("你好，世界", 11, false), "你好，世界");

		assert_eq!(truncate("Hello, world", 13, true), "Hello, world");
		assert_eq!(truncate("你好，世界", 11, true), "你好，世界");
	}
}
