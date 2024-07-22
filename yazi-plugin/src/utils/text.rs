use std::ops::ControlFlow;

use mlua::{Lua, Table};
use unicode_width::UnicodeWidthChar;

use super::Utils;
use crate::CLIPBOARD;

impl Utils {
	pub(super) fn text(lua: &Lua, ya: &Table) -> mlua::Result<()> {
		ya.raw_set(
			"quote",
			lua.create_function(|_, (s, unix): (mlua::String, Option<bool>)| {
				let s = match unix {
					Some(true) => yazi_shared::shell::escape_unix(s.to_str()?),
					Some(false) => yazi_shared::shell::escape_windows(s.to_str()?),
					None => yazi_shared::shell::escape_native(s.to_str()?),
				};
				Ok(s.into_owned())
			})?,
		)?;

		ya.raw_set(
			"truncate",
			lua.create_function(|_, (text, t): (mlua::String, Table)| {
				let (max, text) = (t.raw_get("max")?, text.to_string_lossy());

				Ok(if t.raw_get("rtl").unwrap_or(false) {
					Self::truncate(text.chars().rev(), max).into_iter().rev().collect()
				} else {
					Self::truncate(text.chars(), max).into_iter().collect::<String>()
				})
			})?,
		)?;

		ya.raw_set(
			"clipboard",
			lua.create_async_function(|lua, text: Option<String>| async move {
				if let Some(text) = text {
					CLIPBOARD.set(text).await;
					Ok(None)
				} else {
					Some(lua.create_string(CLIPBOARD.get().await.as_encoded_bytes())).transpose()
				}
			})?,
		)?;

		Ok(())
	}

	fn truncate(mut chars: impl Iterator<Item = char>, max: usize) -> Vec<char> {
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
}
