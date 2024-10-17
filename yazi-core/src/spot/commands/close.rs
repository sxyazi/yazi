use yazi_macro::render;
use yazi_shared::event::Cmd;

use crate::spot::Spot;

struct Opt;

impl From<Cmd> for Opt {
	fn from(_: Cmd) -> Self { Self }
}
impl From<()> for Opt {
	fn from(_: ()) -> Self { Self }
}

impl Spot {
	#[yazi_codegen::command]
	pub fn close(&mut self, _: Opt) {
		self.ct.take().map(|h| h.cancel());
		render!(self.lock.take().is_some());
	}
}
