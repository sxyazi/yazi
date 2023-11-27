use bitflags::bitflags;
use yazi_shared::event::Exec;

use crate::tab::{Mode, Tab};

bitflags! {
	pub struct Opt: u8 {
		const FIND   = 0b0001;
		const VISUAL = 0b0010;
		const SELECT = 0b0100;
		const SEARCH = 0b1000;
	}
}

impl From<&Exec> for Opt {
	fn from(e: &Exec) -> Self {
		e.named.iter().fold(Opt::empty(), |acc, (k, _)| match k.as_bytes() {
			b"all" => Self::all(),
			b"find" => acc | Self::FIND,
			b"visual" => acc | Self::VISUAL,
			b"select" => acc | Self::SELECT,
			b"search" => acc | Self::SEARCH,
			_ => acc,
		})
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
		let opt = opt.into() as Opt;
		if opt.is_empty() {
			return self.escape_find()
				|| self.escape_visual()
				|| self.escape_select()
				|| self.escape_search();
		}

		let mut b = false;
		if opt.contains(Opt::FIND) {
			b |= self.escape_find();
		}
		if opt.contains(Opt::VISUAL) {
			b |= self.escape_visual();
		}
		if opt.contains(Opt::SELECT) {
			b |= self.escape_select();
		}
		if opt.contains(Opt::SEARCH) {
			b |= self.escape_search();
		}
		b
	}
}
