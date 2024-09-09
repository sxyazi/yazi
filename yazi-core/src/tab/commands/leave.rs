use yazi_shared::event::Cmd;

use crate::tab::Tab;

pub struct Opt;
impl From<()> for Opt {
	fn from(_: ()) -> Self { Self }
}
impl From<Cmd> for Opt {
	fn from(_: Cmd) -> Self { Self }
}

impl Tab {
	pub fn leave(&mut self, _: impl Into<Opt>) {
		self
			.current
			.hovered()
			.and_then(|h| h.parent())
			.filter(|p| p != self.cwd())
			.or_else(|| self.cwd().parent_url())
			.map(|u| self.cd(u));
	}
}
