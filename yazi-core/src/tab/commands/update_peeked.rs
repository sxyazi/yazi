use yazi_macro::render;
use yazi_parser::tab::UpdatePeekedOpt;

use crate::tab::Tab;

impl Tab {
	pub fn update_peeked(&mut self, opt: impl TryInto<UpdatePeekedOpt>) {
		let Some(hovered) = self.hovered().map(|h| &h.url) else {
			return self.preview.reset();
		};

		let Ok(opt): Result<UpdatePeekedOpt, _> = opt.try_into() else {
			return;
		};

		if opt.lock.url != *hovered {
			return;
		}

		self.preview.lock = Some(opt.lock);
		render!();
	}
}
