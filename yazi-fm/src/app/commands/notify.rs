use anyhow::Result;
use yazi_macro::succ;
use yazi_proxy::options::NotifyOpt;
use yazi_shared::event::Data;

use crate::app::App;

impl App {
	pub(crate) fn notify(&mut self, opt: NotifyOpt) -> Result<Data> {
		succ!(self.core.notify.push(opt));
	}
}
