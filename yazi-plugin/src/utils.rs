use std::ops::ControlFlow;

use mlua::Table;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::{GLOBALS, LUA};

pub fn init() -> mlua::Result<()> {
	let utils: Table = GLOBALS.get("utils")?;

	utils.set(
		"truncate",
		LUA.create_function(|_, (text, max): (String, usize)| {
			let mut width = 0;
			let flow = text.chars().try_fold(String::with_capacity(max), |mut s, c| {
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
