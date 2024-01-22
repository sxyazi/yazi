use yazi_plugin::utils::PreviewLock;
use yazi_shared::{event::Exec, render};

use crate::tab::Tab;

pub struct Opt {
	lock: PreviewLock,
}

impl TryFrom<Exec> for Opt {
	type Error = ();

	fn try_from(mut e: Exec) -> Result<Self, Self::Error> {
		Ok(Self { lock: e.take_data().ok_or(())? })
	}
}

impl Tab {
	pub fn preview(&mut self, opt: impl TryInto<Opt>) {
		let Some(hovered) = self.current.hovered().map(|h| &h.url) else {
			return render!(self.preview.reset());
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
