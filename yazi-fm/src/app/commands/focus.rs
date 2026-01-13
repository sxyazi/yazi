use anyhow::Result;
use yazi_actor::Ctx;
use yazi_macro::act;
use yazi_parser::VoidOpt;
use yazi_shared::data::Data;

use crate::app::App;

impl App {
	pub fn focus(&mut self, _: VoidOpt) -> Result<Data> {
		let cx = &mut Ctx::active(&mut self.core);

		act!(mgr:refresh, cx)
	}
}
