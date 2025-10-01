use anyhow::Result;
use yazi_config::keymap::Key;
use yazi_macro::{act, render, succ};
use yazi_shared::{data::Data, replace_cow};

use crate::input::{Input, InputMode};

impl Input {
	pub fn r#type(&mut self, key: &Key) -> Result<bool> {
		let Some(c) = key.plain() else { return Ok(false) };

		if self.mode() == InputMode::Insert {
			self.type_str(c.encode_utf8(&mut [0; 4]))?;
			return Ok(true);
		} else if self.mode() == InputMode::Replace {
			self.replace_str(c.encode_utf8(&mut [0; 4]))?;
			return Ok(true);
		}

		Ok(false)
	}

	pub fn type_str(&mut self, s: &str) -> Result<Data> {
		let s = replace_cow(replace_cow(s, "\r", " "), "\n", " ");

		let snap = self.snap_mut();
		if snap.cursor < 1 {
			snap.value.insert_str(0, &s);
		} else {
			snap.value.insert_str(snap.idx(snap.cursor).unwrap(), &s);
		}

		act!(r#move, self, s.chars().count() as isize)?;
		self.flush_value();
		succ!(render!());
	}
}
