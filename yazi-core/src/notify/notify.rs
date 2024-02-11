use std::ops::ControlFlow;

use tokio::task::JoinHandle;
use yazi_shared::term::Term;

use super::{Message, NOTIFY_SPACING};

#[derive(Default)]
pub struct Notify {
	pub(super) tick_handle: Option<JoinHandle<()>>,
	pub messages:           Vec<Message>,
}

impl Notify {
	pub fn limit(&self) -> usize {
		if self.messages.is_empty() {
			return 0;
		}

		let mut height = Term::size().height as usize;
		let flow = (0..self.messages.len().min(3)).try_fold(0, |acc, i| {
			match height.checked_sub(self.messages[i].height() + NOTIFY_SPACING as usize) {
				Some(h) => {
					height = h;
					ControlFlow::Continue(acc + 1)
				}
				None => ControlFlow::Break(acc),
			}
		});

		1.max(match flow {
			ControlFlow::Continue(i) => i,
			ControlFlow::Break(i) => i,
		})
	}
}
