use std::time::Duration;

use yazi_config::keymap::{Exec, KeymapLayer};
use yazi_shared::{Debounce, InputError};
use tokio::pin;
use tokio_stream::{wrappers::UnboundedReceiverStream, StreamExt};

use crate::{emit, input::InputOpt, tab::{Finder, FinderCase, Tab}};

impl Tab {
	pub fn find(&mut self, query: Option<&str>, prev: bool, case: FinderCase) -> bool {
		if let Some(query) = query {
			let Ok(finder) = Finder::new(query, case) else {
				return false;
			};

			let step = if prev {
				finder.prev(&self.current.files, self.current.cursor, true)
			} else {
				finder.next(&self.current.files, self.current.cursor, true)
			};

			if let Some(step) = step {
				self.arrow(step.into());
			}

			self.finder = Some(finder);
			return true;
		}

		tokio::spawn(async move {
			let rx = emit!(Input(
				InputOpt::top(if prev { "Find previous:" } else { "Find next:" }).with_realtime()
			));

			let rx = Debounce::new(UnboundedReceiverStream::new(rx), Duration::from_millis(50));
			pin!(rx);

			while let Some(Ok(s)) | Some(Err(InputError::Typed(s))) = rx.next().await {
				emit!(Call(
					Exec::call("find", vec![s])
						.with_bool("previous", prev)
						.with_bool("smart", case == FinderCase::Smart)
						.with_bool("insensitive", case == FinderCase::Insensitive)
						.vec(),
					KeymapLayer::Manager
				));
			}
		});
		false
	}

	pub fn find_arrow(&mut self, prev: bool) -> bool {
		let Some(finder) = &mut self.finder else {
			return false;
		};

		let b = finder.catchup(&self.current.files);
		let step = if prev {
			finder.prev(&self.current.files, self.current.cursor, false)
		} else {
			finder.next(&self.current.files, self.current.cursor, false)
		};

		b | step.is_some_and(|s| self.arrow(s.into()))
	}
}
