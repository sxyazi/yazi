use anyhow::Result;
use yazi_actor::Ctx;
use yazi_macro::act;
use yazi_parser::VoidOpt;
use yazi_shared::data::Data;

use crate::app::App;

impl App {
	pub fn resize(&mut self, _: VoidOpt) -> Result<Data> {
		act!(reflow, self)?;

		self.core.current_mut().arrow(0);
		self.core.parent_mut().map(|f| f.arrow(0));
		self.core.current_mut().sync_page(true);

		let cx = &mut Ctx::active(&mut self.core);
		act!(mgr:peek, cx)
	}
}
