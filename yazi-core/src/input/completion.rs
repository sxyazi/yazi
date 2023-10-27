use crate::{completion::CompletionOpt, input::Input};

impl Input {
	pub fn complete(&mut self) -> bool {
		if !self.completion.visible {
			eprintln!("Completing 1: {}", self.completion_callback.is_some());
			let current = self.snaps.current().value.clone();
			let result = self.completion_callback.as_ref().map(|f| f(current)).unwrap_or_default();
			eprintln!("Completing {:?}", result);
			match result.len() {
				0 => false,
				1 => self.type_str(result.get(0).unwrap()),
				_ => {
					self.completion.show(CompletionOpt::top().with_items(result));
					true
				}
			}
		} else {
			eprintln!("Completing @");
			self.completion.next(1)
		}
	}
}
