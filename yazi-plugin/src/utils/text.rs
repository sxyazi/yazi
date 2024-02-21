use std::ops::ControlFlow;

use mlua::{Lua, Table};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use super::Utils;

impl Utils {
	pub(super) fn text(lua: &Lua, ya: &Table) -> mlua::Result<()> {
		ya.set(
			"quote",
			lua.create_function(|_, s: mlua::String| {
				#[cfg(unix)]
				let s = shell_escape::unix::escape(s.to_str()?.into());
				#[cfg(windows)]
				let s = shell_escape::windows::escape(s.to_str()?.into());
				Ok(s.into_owned())
			})?,
		)?;

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

		Ok(())
	}
}
