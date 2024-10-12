use yazi_macro::render;
use yazi_shared::event::Cmd;

use crate::manager::Manager;

struct Opt;

impl From<Cmd> for Opt {
	fn from(_: Cmd) -> Self { Self }
}
impl From<()> for Opt {
	fn from(_: ()) -> Self { Self }
}

impl Manager {
	#[yazi_codegen::command]
	pub fn unyank(&mut self, _: Opt) {
		self.yanked.clear();
		render!(self.yanked.catchup_revision(false));
	}
}
