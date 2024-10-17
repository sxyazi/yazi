use yazi_macro::render;
use yazi_plugin::utils::SpotLock;
use yazi_shared::event::Cmd;

use crate::tab::Tab;

pub struct Opt {
	lock: SpotLock,
}

impl TryFrom<Cmd> for Opt {
	type Error = ();

	fn try_from(mut c: Cmd) -> Result<Self, Self::Error> {
		Ok(Self { lock: c.take_any("lock").ok_or(())? })
	}
}

impl Tab {
	pub fn update_spotted(&mut self, opt: impl TryInto<Opt>) {
		let Some(hovered) = self.hovered().map(|h| &h.url) else {
			return self.preview.reset();
		};

		let Ok(opt) = opt.try_into() else {
			return;
		};

		if opt.lock.url != *hovered {
			return;
		}

		self.spot.skip = opt.lock.selected().unwrap_or_default();
		self.spot.lock = Some(opt.lock);
		render!();
	}
}
