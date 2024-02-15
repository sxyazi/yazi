use bitflags::bitflags;
use yazi_shared::{event::Cmd, render};

use crate::{manager::Manager, tab::{Mode, Tab}};

bitflags! {
	pub struct Opt: u8 {
		const FIND   = 0b00001;
		const VISUAL = 0b00010;
		const SELECT = 0b00100;
		const FILTER = 0b01000;
		const SEARCH = 0b10000;
	}
}

impl From<Cmd> for Opt {
	fn from(c: Cmd) -> Self {
		c.named.iter().fold(Opt::empty(), |acc, (k, _)| match k.as_str() {
			"all" => Self::all(),
			"find" => acc | Self::FIND,
			"visual" => acc | Self::VISUAL,
			"select" => acc | Self::SELECT,
			"filter" => acc | Self::FILTER,
			"search" => acc | Self::SEARCH,
			_ => acc,
		})
	}
}

impl Tab {
	#[inline]
	pub fn escape_find(&mut self) -> bool { self.finder.take().is_some() }

	#[inline]
	pub fn escape_visual(&mut self) -> bool {
		let Some((_, indices)) = self.mode.visual() else {
			return false;
		};

		let state = self.mode.is_select();
		for f in indices.iter().filter_map(|i| self.current.files.get(*i)) {
			if state {
				self.selected.insert(f.url());
			} else {
				self.selected.remove(&f.url);
			}
		}

		self.mode = Mode::Normal;
		render!();
		true
	}

	#[inline]
	pub fn escape_select(&mut self) -> bool {
		if self.selected.is_empty() {
			return false;
		}

		self.selected.clear();
		if self.current.hovered().is_some_and(|h| h.is_dir()) {
			Manager::_peek(true);
		}
		true
	}

	#[inline]
	pub fn escape_filter(&mut self) -> bool {
		let b = self.current.files.filter().is_some();
		self.filter_do(super::filter::Opt::default());
		b
	}

	#[inline]
	pub fn escape_search(&mut self) -> bool {
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
