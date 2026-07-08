use anyhow::Result;
use yazi_macro::{act, render, succ};
use yazi_shared::{data::Data, replace_cow};
use yazi_term::event::KeyEvent;

use crate::input::{Input, InputMode};

impl Input {
	pub fn r#type(&mut self, key: &KeyEvent) -> Result<bool> {
		let mut buf = [0; 4];
		let Some(text) = key.text(&mut buf) else { return Ok(false) };

		if self.mode() == InputMode::Insert {
			self.type_str(text)?;
			return Ok(true);
		} else if self.mode() == InputMode::Replace {
			self.replace_str(text)?;
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
		self.flush_all();
		succ!(render!());
	}
}
