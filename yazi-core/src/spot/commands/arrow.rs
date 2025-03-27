use yazi_fs::Step;
use yazi_macro::render;
use yazi_proxy::MgrProxy;
use yazi_shared::event::CmdCow;

use crate::spot::Spot;

struct Opt {
	step: Step,
}

impl From<CmdCow> for Opt {
	fn from(c: CmdCow) -> Self {
		Self { step: c.first().and_then(|d| d.try_into().ok()).unwrap_or_default() }
	}
}

impl Spot {
	#[yazi_codegen::command]
	pub fn arrow(&mut self, opt: Opt) {
		let Some(lock) = &mut self.lock else { return };

		let new = opt.step.add(self.skip, lock.len().unwrap_or(u16::MAX as _), 0);
		let Some(old) = lock.selected() else {
			return MgrProxy::spot(Some(new));
		};

		lock.select(Some(new));
		let new = lock.selected().unwrap();

		self.skip = new;
		render!(new != old);
	}
}
