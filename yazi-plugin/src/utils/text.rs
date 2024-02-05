use std::ops::ControlFlow;

use mlua::{Lua, Table};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};
use yazi_shared::mime_valid;

use super::Utils;

impl Utils {
	pub(super) fn text(lua: &Lua, ya: &Table) -> mlua::Result<()> {
		ya.set(
			"truncate",
			lua.create_function(|_, (text, max): (mlua::String, usize)| {
				let mut width = 0;
				let flow =
					text.to_string_lossy().chars().try_fold(String::with_capacity(max), |mut s, c| {
						width += c.width().unwrap_or(0);
						if s.width() < max {
							s.push(c);
							ControlFlow::Continue(s)
						} else {
							ControlFlow::Break(s)
						}
					});

				Ok(match flow {
					ControlFlow::Break(s) => s,
					ControlFlow::Continue(s) => s,
				})
			})?,
		)?;

		ya.set(
			"mime_valid",
			lua.create_function(|_, mime: mlua::String| Ok(mime_valid(mime.as_bytes())))?,
		)?;

		ya.set(
			"shell_join",
			lua.create_function(|_, table: Table| {
				let mut s = String::new();
				for v in table.sequence_values::<mlua::String>() {
					s.push_str(shell_words::quote(v?.to_str()?).as_ref());
					s.push(' ');
				}
				s.pop();
				Ok(s)
			})?,
		)?;

		Ok(())
	}
}
