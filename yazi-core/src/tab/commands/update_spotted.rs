use yazi_macro::render;
use yazi_plugin::utils::SpotLock;
use yazi_shared::event::CmdCow;

use crate::tab::Tab;

pub struct Opt {
	lock: SpotLock,
}

impl TryFrom<CmdCow> for Opt {
	type Error = ();

	fn try_from(mut c: CmdCow) -> Result<Self, Self::Error> {
		Ok(Self { lock: c.take_any("lock").ok_or(())? })
	}
}

impl Tab {
	pub fn update_spotted(&mut self, opt: impl TryInto<Opt>) {
		let Some(hovered) = self.hovered().map(|h| &h.url) else {
			return self.spot.reset();
		};

		let Ok(mut opt): Result<Opt, _> = opt.try_into() else {
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
