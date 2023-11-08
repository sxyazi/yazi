use std::{mem, time::Duration};

use tokio::pin;
use tokio_stream::{wrappers::UnboundedReceiverStream, StreamExt};
use yazi_config::keymap::{Exec, KeymapLayer};
use yazi_shared::{expand_path, Debounce, InputError, Url};

use crate::{emit, files::{File, FilesOp}, input::InputOpt, tab::Tab};

impl Tab {
	pub fn cd(&mut self, mut target: Url) -> bool {
		let mut hovered = None;
		if let (false, Some(parent)) = (target.pop_slash(), target.parent_url()) {
			emit!(Files(FilesOp::Creating(parent.clone(), File::from_dummy(target.clone()).into_map())));
			hovered = Some(target);
			target = parent;
		}

		// Already in target
		if self.current.cwd == target {
			if let Some(h) = hovered {
				emit!(Hover(h));
			}
			return false;
		}

		// Take parent to history
		if let Some(rep) = self.parent.take() {
			self.history.insert(rep.cwd.clone(), rep);
		}

		// Current
		let rep = self.history_new(&target);
		let rep = mem::replace(&mut self.current, rep);
		if rep.cwd.is_regular() {
			self.history.insert(rep.cwd.clone(), rep);
		}

		// Parent
		if let Some(parent) = target.parent_url() {
			self.parent = Some(self.history_new(&parent));
		}

		// Hover the file
		if let Some(h) = hovered {
			emit!(Hover(h));
		}

		// Backstack
		if target.is_regular() {
			self.backstack.push(target.clone());
		}

		emit!(Refresh);
		true
	}

	pub fn cd_interactive(&mut self, target: Url) -> bool {
		tokio::spawn(async move {
			let rx = emit!(Input(
				InputOpt::top("Change directory:").with_value(target.to_string_lossy()).with_completion()
			));

			let rx = Debounce::new(UnboundedReceiverStream::new(rx), Duration::from_millis(50));
			pin!(rx);

			while let Some(result) = rx.next().await {
				match result {
					Ok(s) => {
						emit!(Call(
							Exec::call("cd", vec![expand_path(s).to_string_lossy().to_string()]).vec(),
							KeymapLayer::Manager
						));
					}
					Err(InputError::Completed(before, ticket)) => {
						emit!(Call(
							Exec::call("complete", vec![]).with("before", before).with("ticket", ticket).vec(),
							KeymapLayer::Input
						));
					}
					_ => break,
				}
			}
		});
		false
	}
}
