use yazi_shared::event::CmdCow;

use super::cd::CdSource;
use crate::tab::Tab;

struct Opt;
impl From<()> for Opt {
	fn from(_: ()) -> Self { Self }
}
impl From<CmdCow> for Opt {
	fn from(_: CmdCow) -> Self { Self }
}

impl Tab {
	#[yazi_codegen::command]
	pub fn leave(&mut self, _: Opt) {
		self
			.current
			.hovered()
			.and_then(|h| h.url.parent_url())
			.filter(|u| u != self.cwd())
			.or_else(|| self.cwd().parent_url())
			.map(|u| self.cd((u.into_regular(), CdSource::Leave)));
	}
}
