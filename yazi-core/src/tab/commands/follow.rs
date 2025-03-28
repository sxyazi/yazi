use yazi_fs::clean_url;
use yazi_shared::event::CmdCow;

use crate::tab::Tab;

struct Opt;

impl From<CmdCow> for Opt {
	fn from(_: CmdCow) -> Self { Self }
}

impl Tab {
	#[yazi_codegen::command]
	pub fn follow(&mut self, _: Opt) {
		let Some(file) = self.hovered() else { return };
		let Some(link_to) = &file.link_to else { return };

		if link_to.is_absolute() {
			self.reveal(link_to.to_owned());
		} else if let Some(p) = file.url.parent_url() {
			self.reveal(clean_url(&p.join(link_to)));
		}
	}
}
