use std::borrow::Cow;

use yazi_proxy::{MgrProxy, TabProxy};
use yazi_shared::event::CmdCow;

use crate::spot::Spot;

struct Opt {
	step: Cow<'static, str>,
}

impl From<CmdCow> for Opt {
	fn from(mut c: CmdCow) -> Self { Self { step: c.take_first_str().unwrap_or_default() } }
}

impl Spot {
	#[yazi_codegen::command]
	pub fn swipe(&mut self, opt: Opt) {
		TabProxy::arrow(opt.step);
		MgrProxy::spot(None);
	}
}
