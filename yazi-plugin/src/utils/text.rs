use mlua::{Function, Lua, Table};
use twox_hash::XxHash3_128;
use unicode_width::UnicodeWidthChar;
use yazi_binding::deprecate;
use yazi_widgets::CLIPBOARD;

use super::Utils;

impl Utils {
	pub(super) fn hash(lua: &Lua) -> mlua::Result<Function> {
		lua.create_async_function(move |_, s: mlua::String| async move {
			Ok(format!("{:x}", XxHash3_128::oneshot(&s.as_bytes())))
		})
	}

	pub(super) fn quote(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|lua, (s, unix): (mlua::String, Option<bool>)| {
			let b = s.as_bytes();
			let s = match unix {
				Some(true) => yazi_shared::shell::unix::escape_os_bytes(&b),
				Some(false) => yazi_shared::shell::windows::escape_os_bytes(&b),
				None => yazi_shared::shell::escape_os_bytes(&b),
			};
			lua.create_string(&*s)
		})
	}

	pub(super) fn truncate(lua: &Lua) -> mlua::Result<Function> {
		fn traverse(
			it: impl Iterator<Item = (usize, char)>,
			max: usize,
		) -> (Option<usize>, usize, bool) {
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

		lua.create_function(|lua, (s, t): (mlua::String, Table)| {
			deprecate!(lua, "`ya.truncate()` is deprecated, use `ui.truncate()` instead, in your {}\nSee #2939 for more details: https://github.com/sxyazi/yazi/pull/2939");

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
		})
	}

	pub(super) fn clipboard(lua: &Lua) -> mlua::Result<Function> {
		lua.create_async_function(|lua, text: Option<String>| async move {
			if let Some(text) = text {
				CLIPBOARD.set(text).await;
				Ok(None)
			} else {
				Some(lua.create_string(CLIPBOARD.get().await)).transpose()
			}
		})
	}
}
