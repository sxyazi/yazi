use yazi_parser::spot::SwipeOpt;
use yazi_proxy::{MgrProxy, TabProxy};

use crate::spot::Spot;

impl Spot {
	#[yazi_codegen::command]
	pub fn swipe(&mut self, opt: SwipeOpt) {
		TabProxy::arrow(opt.step);
		MgrProxy::spot(None);
	}
}
