use crossterm::event::KeyCode;
use yazi_config::keymap::Key;

use crate::{completion::CompletionOpt, input::Input};

impl Input {
	pub fn complete(&mut self) -> bool {
		if !self.completion.visible {
			let current = self.snaps.current().value.as_str();
			let result = self.init_completion.as_ref().map(|f| f(current)).unwrap_or_default();
			match result.len() {
				0 => false,
				1 => {
					if let Some(f) = self.finish_completion.as_ref() {
						self.replace_str(f(current, result.get(0).unwrap()).as_str())
					} else {
						false
					}
				}
				_ => {
					self.completion.show(CompletionOpt::top().with_items(result));
					true
				}
			}
		} else {
			false
		}
	}

	pub fn navigate_completion(&mut self, key: &Key) -> bool {
		match key.code {
			KeyCode::Up => self.completion.prev(self.completion.column_cnt as usize),
			KeyCode::Down => self.completion.next(self.completion.column_cnt as usize),
			KeyCode::Left => self.completion.prev(1),
			KeyCode::Right | KeyCode::Tab => self.completion.next(1),
			_ => false,
		}
	}

	pub fn finish_completion(&mut self) -> bool {
		if let (Some(val), Some(f)) = (self.completion.get_selection(), &self.finish_completion) {
			let final_val = f(self.snaps.current().value.as_str(), val.as_str());
			self.replace_str(final_val.as_str());
		}
		self.completion.close()
	}
}
