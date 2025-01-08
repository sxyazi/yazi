use std::ops::ControlFlow;

use md5::{Digest, Md5};
use mlua::{Function, Lua, Table};
use twox_hash::XxHash3_128;
use unicode_width::UnicodeWidthChar;

use super::Utils;
use crate::CLIPBOARD;

impl Utils {
	pub(super) fn hash(lua: &Lua, deprecated: bool) -> mlua::Result<Function> {
		lua.create_async_function(move |lua, s: mlua::String| async move {
			if deprecated {
				crate::deprecate!(
					lua,
					"The `ya.md5()` function is deprecated, please use `ya.hash()` instead, in your {}"
				);
				Ok(format!("{:x}", Md5::new_with_prefix(s.as_bytes()).finalize()))
			} else {
				Ok(format!("{:x}", XxHash3_128::oneshot(s.as_bytes().as_ref())))
			}
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
		fn truncate_impl(mut chars: impl Iterator<Item = char>, max: usize) -> Vec<char> {
			let mut width = 0;
			let flow = chars.try_fold(Vec::with_capacity(max), |mut v, c| {
				width += c.width().unwrap_or(0);
				if width < max {
					v.push(c);
					ControlFlow::Continue(v)
				} else {
					ControlFlow::Break(v)
				}
			});

			match flow {
				ControlFlow::Break(v) => v,
				ControlFlow::Continue(v) => v,
			}
		}

		lua.create_function(|_, (text, t): (mlua::String, Table)| {
			let (max, text) = (t.raw_get("max")?, text.to_string_lossy());

			Ok(if t.raw_get("rtl").unwrap_or(false) {
				truncate_impl(text.chars().rev(), max).into_iter().rev().collect()
			} else {
				truncate_impl(text.chars(), max).into_iter().collect::<String>()
			})
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
