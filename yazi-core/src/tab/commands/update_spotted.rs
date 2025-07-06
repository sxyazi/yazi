use yazi_macro::render;
use yazi_parser::tab::UpdateSpottedOpt;

use crate::tab::Tab;

impl Tab {
	pub fn update_spotted(&mut self, opt: impl TryInto<UpdateSpottedOpt>) {
		let Some(hovered) = self.hovered().map(|h| &h.url) else {
			return self.spot.reset();
		};

		let Ok(mut opt): Result<UpdateSpottedOpt, _> = opt.try_into() else {
			return;
		};

		if opt.lock.url != *hovered {
			return;
		}

		if self.spot.lock.as_ref().is_none_or(|l| l.id != opt.lock.id) {
			self.spot.skip = opt.lock.selected().unwrap_or_default();
		} else if let Some(s) = opt.lock.selected() {
			self.spot.skip = s;
		} else {
			opt.lock.select(Some(self.spot.skip));
		}

		self.spot.lock = Some(opt.lock);
		render!();
	}
}
