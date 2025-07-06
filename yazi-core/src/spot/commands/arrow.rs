use yazi_macro::render;
use yazi_parser::spot::ArrowOpt;
use yazi_proxy::MgrProxy;

use crate::spot::Spot;

impl Spot {
	#[yazi_codegen::command]
	pub fn arrow(&mut self, opt: ArrowOpt) {
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
