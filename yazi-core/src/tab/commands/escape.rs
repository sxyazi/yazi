use bitflags::bitflags;
use yazi_macro::{render, render_and};
use yazi_proxy::{AppProxy, MgrProxy};
use yazi_shared::event::CmdCow;

use crate::tab::Tab;

bitflags! {
	struct Opt: u8 {
		const FIND   = 0b00001;
		const VISUAL = 0b00010;
		const FILTER = 0b00100;
		const SELECT = 0b01000;
		const SEARCH = 0b10000;
	}
}

impl From<CmdCow> for Opt {
	fn from(c: CmdCow) -> Self {
		c.args.iter().fold(Opt::empty(), |acc, (k, v)| {
			match (k.as_str().unwrap_or(""), v.as_bool().unwrap_or(false)) {
				("all", true) => Self::all(),
				("find", true) => acc | Self::FIND,
				("visual", true) => acc | Self::VISUAL,
				("filter", true) => acc | Self::FILTER,
				("select", true) => acc | Self::SELECT,
				("search", true) => acc | Self::SEARCH,
				_ => acc,
			}
		})
	}
}

impl Tab {
	#[yazi_codegen::command]
	pub fn escape(&mut self, opt: Opt) {
		if opt.is_empty() {
			_ = self.escape_find()
				|| self.escape_visual()
				|| self.escape_filter()
				|| self.escape_select()
				|| self.escape_search();
			return;
		}

		if opt.contains(Opt::FIND) {
			self.escape_find();
		}
		if opt.contains(Opt::VISUAL) {
			self.escape_visual();
		}
		if opt.contains(Opt::FILTER) {
			self.escape_filter();
		}
		if opt.contains(Opt::SELECT) {
			self.escape_select();
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

	pub fn escape_filter(&mut self) -> bool {
		if self.current.files.filter().is_none() {
			return false;
		}

		self.filter_do(super::filter::Opt::default());
		render_and!(true)
	}

	pub fn escape_select(&mut self) -> bool {
		if self.selected.is_empty() {
			return false;
		}

		self.selected.clear();
		if self.hovered().is_some_and(|h| h.is_dir()) {
			MgrProxy::peek(true);
		}
		render_and!(true)
	}

	pub fn escape_search(&mut self) -> bool {
		let b = self.cwd().is_search();
		self.search_stop();

		render_and!(b)
	}

	pub fn try_escape_visual(&mut self) -> bool {
		let select = self.mode.is_select();
		let Some((_, indices)) = self.mode.take_visual() else {
			return true;
		};

		render!();
		let urls: Vec<_> =
			indices.into_iter().filter_map(|i| self.current.files.get(i)).map(|f| &f.url).collect();

		if !select {
			self.selected.remove_many(&urls);
		} else if self.selected.add_many(&urls) != urls.len() {
			AppProxy::notify_warn(
				"Escape visual mode",
				"Some files cannot be selected, due to path nesting conflict.",
			);
			return false;
		}

		true
	}
}
