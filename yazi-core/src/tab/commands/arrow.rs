use yazi_dds::Pubsub;
use yazi_proxy::ManagerProxy;
use yazi_shared::{event::Cmd, render};

use crate::{tab::Tab, Step};

pub struct Opt {
	step: Step,
}

impl From<Cmd> for Opt {
	fn from(mut c: Cmd) -> Self {
		Self { step: c.take_first_str().and_then(|s| s.parse().ok()).unwrap_or_default() }
	}
}

impl<T> From<T> for Opt
where
	T: Into<Step>,
{
	fn from(t: T) -> Self { Self { step: t.into() } }
}

impl Tab {
	pub fn arrow(&mut self, opt: impl Into<Opt>) {
		let opt = opt.into() as Opt;
		if !self.current.arrow(opt.step) {
			return;
		}

		// Visual selection
		if let Some((start, items)) = self.mode.visual_mut() {
			let after = self.current.cursor;

			items.clear();
			for i in start.min(after)..=after.max(start) {
				items.insert(i);
			}
		}

		Pubsub::pub_from_hover(self.idx, self.current.hovered().map(|h| &h.url));
		ManagerProxy::hover(None);
		render!();
	}
}
