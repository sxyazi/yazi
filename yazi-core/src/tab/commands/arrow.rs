use yazi_config::keymap::Exec;

use crate::{emit, tab::Tab, Step};

pub struct Opt(Step);

impl From<&Exec> for Opt {
	fn from(e: &Exec) -> Self {
		Self(e.args.first().and_then(|s| s.parse().ok()).unwrap_or_default())
	}
}

impl<T> From<T> for Opt
where
	T: Into<Step>,
{
	fn from(t: T) -> Self { Self(t.into()) }
}

impl Tab {
	pub fn arrow(&mut self, opt: impl Into<Opt>) -> bool {
		let step = opt.into().0;
		let ok = if step.is_positive() { self.current.next(step) } else { self.current.prev(step) };
		if !ok {
			return false;
		}

		// Visual selection
		if let Some((start, items)) = self.mode.visual_mut() {
			let after = self.current.cursor;

			items.clear();
			for i in start.min(after)..=after.max(start) {
				items.insert(i);
			}
		}

		emit!(Hover);
		true
	}
}
