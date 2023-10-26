use crate::{completion::CompletionOpt, input::Input};

impl Input {
	pub fn complete(&mut self) -> bool {
		if !self.completion.visible {
			let current = self.snaps.current().value.clone();
			let result = self.completion_callback.as_ref().map(|f| f(current)).unwrap_or_default();
			match result.len() {
				0 => false,
				1 => self.type_str(result.get(0).unwrap()),
				_ => {
					self.completion.show(CompletionOpt::hovered().with_items(result));
					false
				}
			}
		} else {
			self.completion.next(1)
		}
	}
}
