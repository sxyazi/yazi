use bitflags::bitflags;
use yazi_shared::{event::Exec, render};

use crate::tab::{Mode, Tab};

bitflags! {
	pub struct Opt: u8 {
		const FIND   = 0b00001;
		const VISUAL = 0b00010;
		const SELECT = 0b00100;
		const FILTER = 0b01000;
		const SEARCH = 0b10000;
	}
}

impl From<&Exec> for Opt {
	fn from(e: &Exec) -> Self {
		e.named.iter().fold(Opt::empty(), |acc, (k, _)| match k.as_bytes() {
			b"all" => Self::all(),
			b"find" => acc | Self::FIND,
			b"visual" => acc | Self::VISUAL,
			b"select" => acc | Self::SELECT,
			b"filter" => acc | Self::FILTER,
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
	fn escape_select(&mut self) -> bool { self.current.files.select_all(Some(false)) }

	#[inline]
	fn escape_filter(&mut self) -> bool {
		let b = self.current.files.filter().is_some();
		self.filter_do(super::filter::Opt { query: "", ..Default::default() });
		b
	}

	#[inline]
	fn escape_search(&mut self) -> bool {
		let b = self.current.cwd.is_search();
		self.search_stop();
		b
	}

	pub fn escape(&mut self, opt: impl Into<Opt>) {
		let opt = opt.into() as Opt;
		if opt.is_empty() {
			return render!(
				self.escape_find()
					|| self.escape_visual()
					|| self.escape_select()
					|| self.escape_filter()
					|| self.escape_search()
			);
		}

		if opt.contains(Opt::FIND) {
			render!(self.escape_find());
		}
		if opt.contains(Opt::VISUAL) {
			render!(self.escape_visual());
		}
		if opt.contains(Opt::SELECT) {
			render!(self.escape_select());
		}
		if opt.contains(Opt::FILTER) {
			render!(self.escape_filter());
		}
		if opt.contains(Opt::SEARCH) {
			render!(self.escape_search());
		}
	}
}
