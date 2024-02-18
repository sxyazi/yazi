use bitflags::bitflags;
use yazi_shared::{event::Cmd, render, render_and};

use crate::{manager::Manager, notify::Notify, tab::Tab};

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
	pub fn escape(&mut self, opt: impl Into<Opt>) {
		let opt = opt.into() as Opt;
		if opt.is_empty() {
			_ = self.escape_find()
				|| self.escape_visual()
				|| self.escape_select()
				|| self.escape_filter()
				|| self.escape_search();
			return;
		}

		if opt.contains(Opt::FIND) {
			self.escape_find();
		}
		if opt.contains(Opt::VISUAL) {
			self.escape_visual();
		}
		if opt.contains(Opt::SELECT) {
			self.escape_select();
		}
		if opt.contains(Opt::FILTER) {
			self.escape_filter();
		}
		if opt.contains(Opt::SEARCH) {
			self.escape_search();
		}
	}

	pub fn escape_find(&mut self) -> bool { render_and!(self.finder.take().is_some()) }

	pub fn escape_visual(&mut self) -> bool {
		if !self.mode.is_visual() {
			return false;
		}

		self.try_escape_visual();
		true
	}

	pub fn escape_select(&mut self) -> bool {
		if self.selected.is_empty() {
			return false;
		}

		self.selected.clear();
		if self.current.hovered().is_some_and(|h| h.is_dir()) {
			Manager::_peek(true);
		}
		render_and!(true)
	}

	pub fn escape_filter(&mut self) -> bool {
		if self.current.files.filter().is_none() {
			return false;
		}

		self.filter_do(super::filter::Opt::default());
		render_and!(true)
	}

	pub fn escape_search(&mut self) -> bool {
		if !self.current.cwd.is_search() {
			return false;
		}

		self.search_stop();
		render_and!(true)
	}

	pub fn try_escape_visual(&mut self) -> bool {
		let state = self.mode.is_select();
		let Some((_, indices)) = self.mode.take_visual() else {
			return true;
		};

		let results: Vec<_> = indices
			.iter()
			.filter_map(|i| self.current.files.get(*i))
			.map(|f| {
				if state {
					self.selected.add(&f.url)
				} else {
					self.selected.remove(&f.url);
					true
				}
			})
			.collect();

		render!(!results.is_empty());
		if results.into_iter().all(|b| b) {
			return true;
		}

		Notify::_push_warn(
			"Escape visual mode",
			"Some files cannot be selected due to path nesting conflict.",
		);
		false
	}
}
