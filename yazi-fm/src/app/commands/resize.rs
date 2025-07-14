use anyhow::Result;
use yazi_macro::{act, succ};
use yazi_parser::VoidOpt;
use yazi_shared::event::Data;

use crate::app::App;

impl App {
	pub fn resize(&mut self, _: VoidOpt) -> Result<Data> {
		act!(reflow, self)?;

		self.core.current_mut().sync_page(true);
		self.core.current_mut().arrow(0);
		// FIXME
		// self.core.mgr.peek(false);
		self.core.mgr.parent_mut().map(|f| f.arrow(0));

		succ!();
	}
}
