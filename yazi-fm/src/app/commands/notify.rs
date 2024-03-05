use yazi_proxy::options::NotifyOpt;

use crate::app::App;

impl App {
	pub(crate) fn notify(&mut self, opt: impl TryInto<NotifyOpt>) {
		let Ok(opt) = opt.try_into() else {
			return;
		};

		self.cx.notify.push(opt);
	}
}
