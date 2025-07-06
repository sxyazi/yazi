use yazi_macro::{render, render_and};
use yazi_parser::tab::{EscapeOpt, FilterOpt};
use yazi_proxy::{AppProxy, MgrProxy};

use crate::tab::Tab;

impl Tab {
	#[yazi_codegen::command]
	pub fn escape(&mut self, opt: EscapeOpt) {
		if opt.is_empty() {
			_ = self.escape_find()
				|| self.escape_visual()
				|| self.escape_filter()
				|| self.escape_select()
				|| self.escape_search();
			return;
		}

		if opt.contains(EscapeOpt::FIND) {
			self.escape_find();
		}
		if opt.contains(EscapeOpt::VISUAL) {
			self.escape_visual();
		}
		if opt.contains(EscapeOpt::FILTER) {
			self.escape_filter();
		}
		if opt.contains(EscapeOpt::SELECT) {
			self.escape_select();
		}
		if opt.contains(EscapeOpt::SEARCH) {
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

		self.filter_do(FilterOpt::default());
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
