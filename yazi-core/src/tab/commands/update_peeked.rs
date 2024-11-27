use yazi_macro::render;
use yazi_plugin::utils::PreviewLock;
use yazi_shared::event::CmdCow;

use crate::tab::Tab;

pub struct Opt {
	lock: PreviewLock,
}

impl TryFrom<CmdCow> for Opt {
	type Error = ();

	fn try_from(mut c: CmdCow) -> Result<Self, Self::Error> {
		Ok(Self { lock: c.take_any("lock").ok_or(())? })
	}
}

impl Tab {
	pub fn update_peeked(&mut self, opt: impl TryInto<Opt>) {
		let Some(hovered) = self.hovered().map(|h| &h.url) else {
			return self.preview.reset();
		};

		let Ok(opt) = opt.try_into() else {
			return;
		};

		if opt.lock.url != *hovered {
			return;
		}

		self.preview.lock = Some(opt.lock);
		render!();
	}
}
