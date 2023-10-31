use crossterm::event::KeyCode;
use yazi_config::keymap::{Exec, Key, KeymapLayer};

use crate::{completion::CompletionOpt, emit, input::Input};

impl Input {
	pub fn complete(&mut self) -> bool {
		if !self.completion.visible {
			if let Some(f) = self.init_completion.as_ref() {
				let future = f(self.snaps.current().value.clone());
				let id = self.completion.identifier.clone();
				tokio::spawn(async move {
					let result = future.await;
					let mut exec = Exec::call("complete", result);
					exec.named.insert("identifier".to_string(), id);
					emit!(Call(exec.vec(), KeymapLayer::Input));
				});
			}
		}
		false
	}

	pub fn fill_completion(&mut self, exec: &Exec) -> bool {
		if !exec.named.get("identifier").is_some_and(|id| *id == self.completion.identifier) {
			return false;
		};
		match exec.args.len() {
			0 => false,
			1 => {
				if let Some(f) = self.finish_completion.as_ref() {
					self
						.replace_str(f(self.snaps.current().value.as_str(), exec.args.get(0).unwrap()).as_str())
				} else {
					false
				}
			}
			_ => {
				self.completion.show(CompletionOpt::top().with_items(exec.args.clone()));
				true
			}
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
