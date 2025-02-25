use yazi_macro::render;
use yazi_shared::event::CmdCow;

use crate::mgr::Mgr;

struct Opt;

impl From<CmdCow> for Opt {
	fn from(_: CmdCow) -> Self { Self }
}
impl From<()> for Opt {
	fn from(_: ()) -> Self { Self }
}

impl Mgr {
	#[yazi_codegen::command]
	pub fn unyank(&mut self, _: Opt) {
		let repeek = self.hovered().is_some_and(|f| f.is_dir() && self.yanked.contains_in(&f.url));
		self.yanked.clear();

		render!(self.yanked.catchup_revision(false));
		if repeek {
			self.peek(true);
		}
	}
}
