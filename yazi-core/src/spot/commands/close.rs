use yazi_shared::event::CmdCow;

use crate::spot::Spot;

struct Opt;

impl From<CmdCow> for Opt {
	fn from(_: CmdCow) -> Self { Self }
}
impl From<()> for Opt {
	fn from(_: ()) -> Self { Self }
}

impl Spot {
	#[yazi_codegen::command]
	pub fn close(&mut self, _: Opt) { self.reset(); }
}
