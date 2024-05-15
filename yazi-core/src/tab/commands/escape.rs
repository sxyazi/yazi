use bitflags::bitflags;
use yazi_proxy::{AppProxy, ManagerProxy};
use yazi_shared::{
	event::{Cmd, Event, EventQuit},
	render, render_and,
};

use crate::tab::Tab;

bitflags! {
	pub struct Opt: u8 {
		const FIND    = 0b0000001;
		const VISUAL  = 0b0000010;
		const SELECT  = 0b0000100;
		const FILTER  = 0b0001000;
		const SEARCH  = 0b0010000;
		const QUIT    = 0b0100000;
		const CASCADE = 0b1000000;
	}
}

impl From<Cmd> for Opt {
	fn from(c: Cmd) -> Self {
		c.args.iter().fold(Opt::empty(), |acc, (k, v)| {
			match (k.as_str(), v.as_bool().unwrap_or(false)) {
				("all", true) => acc | (Self::all() ^ Self::QUIT ^ Self::CASCADE),
				("find", true) => acc | Self::FIND,
				("visual", true) => acc | Self::VISUAL,
				("select", true) => acc | Self::SELECT,
				("filter", true) => acc | Self::FILTER,
				("search", true) => acc | Self::SEARCH,
				("quit", true) => acc | Self::QUIT,
				("cascade", true) => acc | Self::CASCADE,
				_ => acc,
			}
		})
	}
}

impl Tab {
	pub fn escape(&mut self, opt: impl Into<Opt>) {
		let opt = opt.into() as Opt;
		if opt.is_empty() || opt.contains(Opt::CASCADE) {
			_ = self.escape_find()
				|| self.escape_visual()
				|| self.escape_select()
				|| self.escape_filter()
				|| self.escape_search()
				|| (opt.contains(Opt::QUIT) && self.quit());
			return;
		}

		let mut matched = false;

		if opt.contains(Opt::FIND) {
			matched |= self.escape_find();
		}
		if opt.contains(Opt::VISUAL) {
			matched |= self.escape_visual();
		}
		if opt.contains(Opt::SELECT) {
			matched |= self.escape_select();
		}
		if opt.contains(Opt::FILTER) {
			matched |= self.escape_filter();
		}
		if opt.contains(Opt::SEARCH) {
			matched |= self.escape_search();
		}

		if !matched && opt.contains(Opt::QUIT) {
			self.quit();
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
			ManagerProxy::peek(true);
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
		let select = self.mode.is_select();
		let Some((_, indices)) = self.mode.take_visual() else {
			return true;
		};

		render!();
		let urls: Vec<_> =
			indices.into_iter().filter_map(|i| self.current.files.get(i)).map(|f| &f.url).collect();

		let same = !self.current.cwd.is_search();
		if !select {
			self.selected.remove_many(&urls, same);
		} else if self.selected.add_many(&urls, same) != urls.len() {
			AppProxy::notify_warn(
				"Escape visual mode",
				"Some files cannot be selected, due to path nesting conflict.",
			);
			return false;
		}

		true
	}

	pub fn quit(&mut self) -> bool {
		Event::Quit(EventQuit { no_cwd_file: false, selected: None }).emit();
		true
	}
}
