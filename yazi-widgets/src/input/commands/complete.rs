use std::{borrow::Cow, path::MAIN_SEPARATOR_STR};

use yazi_macro::render;
use yazi_shared::event::{CmdCow, Data};

use crate::input::Input;

#[cfg(windows)]
const SEPARATOR: [char; 2] = ['/', '\\'];

#[cfg(not(windows))]
const SEPARATOR: char = std::path::MAIN_SEPARATOR;

struct Opt {
	word:    Cow<'static, str>,
	_ticket: usize, // FIXME: not used
}

impl From<CmdCow> for Opt {
	fn from(mut c: CmdCow) -> Self {
		Self {
			word:    c.take_first_str().unwrap_or_default(),
			_ticket: c.get("ticket").and_then(Data::as_usize).unwrap_or(0),
		}
	}
}

impl Input {
	#[yazi_codegen::command]
	pub fn complete(&mut self, opt: Opt) {
		let (before, after) = self.partition();
		let new = if let Some((prefix, _)) = before.rsplit_once(SEPARATOR) {
			format!("{prefix}/{}{after}", opt.word).replace(SEPARATOR, MAIN_SEPARATOR_STR)
		} else {
			format!("{}{after}", opt.word).replace(SEPARATOR, MAIN_SEPARATOR_STR)
		};

		let snap = self.snap_mut();
		if new == snap.value {
			return;
		}

		let delta = new.chars().count() as isize - snap.count() as isize;
		snap.value = new;

		self.move_(delta);
		self.flush_value();
		render!();
	}
}
