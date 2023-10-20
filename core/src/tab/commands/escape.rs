use config::keymap::Exec;

use crate::tab::{Mode, Tab};

pub struct Opt(u8);

impl From<&Exec> for Opt {
	fn from(e: &Exec) -> Self {
		Self(e.named.iter().fold(0, |acc, (k, _)| match k.as_str() {
			"all" => 0b1111,
			"find" => acc | 0b0001,
			"visual" => acc | 0b0010,
			"select" => acc | 0b0100,
			"search" => acc | 0b1000,
			_ => acc,
		}))
	}
}

impl Tab {
	#[inline]
	fn escape_find(&mut self) -> bool { self.finder.take().is_some() }

	#[inline]
	fn escape_visual(&mut self) -> bool {
		if let Some((_, indices)) = self.mode.visual() {
			self.current.files.select_index(indices, Some(self.mode.is_select()));
			self.mode = Mode::Normal;
			return true;
		}
		false
	}

	#[inline]
	fn escape_select(&mut self) -> bool { self.select_all(Some(false)) }

	#[inline]
	fn escape_search(&mut self) -> bool { self.search_stop() }

	pub fn escape(&mut self, opt: impl Into<Opt>) -> bool {
		let opt = opt.into().0;
		if opt == 0 {
			return self.escape_find()
				|| self.escape_visual()
				|| self.escape_select()
				|| self.escape_search();
		}

		let mut b = false;
		if opt & 0b0001 != 0 {
			b |= self.escape_find();
		}
		if opt & 0b0010 != 0 {
			b |= self.escape_visual();
		}
		if opt & 0b0100 != 0 {
			b |= self.escape_select();
		}
		if opt & 0b1000 != 0 {
			b |= self.escape_search();
		}
		b
	}
}
