use crate::{emit, tab::Tab, Step};

impl Tab {
	pub fn arrow(&mut self, step: Step) -> bool {
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
