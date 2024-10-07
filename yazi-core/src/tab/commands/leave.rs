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
	#[yazi_macro::command]
	pub fn leave(&mut self, _: Opt) {
		self
			.current
			.hovered()
			.and_then(|h| h.url.parent_url())
			.filter(|u| u != self.cwd())
			.or_else(|| self.cwd().parent_url())
			.map(|u| self.cd(u.into_regular()));
	}
}
