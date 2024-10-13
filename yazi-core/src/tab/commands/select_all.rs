use yazi_shared::event::Cmd;

use crate::tab::Tab;

struct Opt;

impl From<Cmd> for Opt {
	fn from(_: Cmd) -> Self { Self }
}

impl Tab {
	// TODO: remove this in Yazi 0.4.1
	#[yazi_codegen::command]
	pub fn select_all(&mut self, _: Opt) { self.select(()); }
}
