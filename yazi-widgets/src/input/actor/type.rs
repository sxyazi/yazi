use anyhow::Result;
use yazi_macro::succ;
use yazi_shared::data::Data;
use yazi_term::event::KeyEvent;

use crate::input::{Input, InputMode, parser::FeedOpt};

impl Input {
	pub fn r#type(&mut self, key: &KeyEvent) -> Result<bool> {
		let mut buf = [0; 4];
		if self.mode() == InputMode::Normal {
			Ok(false)
		} else if let Some(text) = key.text(&mut buf) {
			Ok(self.feed(text.into()).is_ok())
		} else {
			Ok(false)
		}
	}

	pub fn feed<'a>(&mut self, opt: FeedOpt<'a>) -> Result<Data> {
		match self.mode() {
			InputMode::Normal => succ!(),
			InputMode::Insert => self.insert_str(opt.as_ref()),
			InputMode::Replace => self.replace_str(opt.as_ref()),
		}
	}
}
