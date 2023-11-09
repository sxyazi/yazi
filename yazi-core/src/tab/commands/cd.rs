use std::{mem, time::Duration};

use tokio::{fs, pin};
use tokio_stream::{wrappers::UnboundedReceiverStream, StreamExt};
use yazi_config::keymap::{Exec, KeymapLayer};
use yazi_shared::{expand_path, Debounce, InputError, Url};

use crate::{emit, input::InputOpt, tab::Tab};

impl Tab {
	pub fn cd(&mut self, target: Url) -> bool {
		if self.current.cwd == target {
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
						let p = expand_path(s);
						let Ok(meta) = fs::metadata(&p).await else {
							return;
						};

						emit!(Call(
							Exec::call(if meta.is_dir() { "cd" } else { "reveal" }, vec![
								p.to_string_lossy().to_string()
							])
							.vec(),
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
