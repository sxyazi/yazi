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
				Some(lua.create_string(CLIPBOARD.get().await.as_encoded_bytes())).transpose()
			}
		})
	}
}

#[cfg(test)]
mod tests {
	use mlua::chunk;

	use super::*;

	fn truncate(s: &str, max: usize, rtl: bool) -> String {
		let lua = Lua::new();
		let f = Utils::truncate(&lua).unwrap();

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
