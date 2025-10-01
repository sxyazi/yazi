use anyhow::Result;
use yazi_macro::succ;
use yazi_parser::app::NotifyOpt;
use yazi_shared::data::Data;

use crate::app::App;

impl App {
	pub(crate) fn notify(&mut self, opt: NotifyOpt) -> Result<Data> {
		succ!(self.core.notify.push(opt));
	}
}
