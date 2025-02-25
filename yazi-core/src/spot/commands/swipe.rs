use yazi_proxy::{MgrProxy, TabProxy};
use yazi_shared::event::{CmdCow, Data};

use crate::spot::Spot;

struct Opt {
	step: isize,
}

impl From<CmdCow> for Opt {
	fn from(c: CmdCow) -> Self { Self { step: c.first().and_then(Data::as_isize).unwrap_or(0) } }
}

impl Spot {
	#[yazi_codegen::command]
	pub fn swipe(&mut self, opt: Opt) {
		TabProxy::arrow(opt.step);
		MgrProxy::spot(None);
	}
}
