use yazi_parser::tab::{CdSource, LeaveOpt};

use crate::tab::Tab;

impl Tab {
	#[yazi_codegen::command]
	pub fn leave(&mut self, _: LeaveOpt) {
		self
			.current
			.hovered()
			.and_then(|h| h.url.parent_url())
			.filter(|u| u != self.cwd())
			.or_else(|| self.cwd().parent_url())
			.map(|u| self.cd((u.into_regular(), CdSource::Leave)));
	}
}
