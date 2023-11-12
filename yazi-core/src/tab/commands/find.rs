use std::time::Duration;

use tokio::pin;
use tokio_stream::{wrappers::UnboundedReceiverStream, StreamExt};
use yazi_config::{keymap::{Exec, KeymapLayer}, INPUTBOX};
use yazi_shared::{Debounce, InputError};

use crate::{emit, input::InputOpt, tab::{Finder, FinderCase, Tab}};

pub struct Opt<'a> {
	query: Option<&'a str>,
	prev:  bool,
	case:  FinderCase,
}

impl<'a> From<&'a Exec> for Opt<'a> {
	fn from(e: &'a Exec) -> Self {
		Self {
			query: e.args.first().map(|s| s.as_str()),
			prev:  e.named.contains_key("previous"),
			case:  match (e.named.contains_key("smart"), e.named.contains_key("insensitive")) {
				(true, _) => FinderCase::Smart,
				(_, false) => FinderCase::Sensitive,
				(_, true) => FinderCase::Insensitive,
			},
		}
	}
}

pub struct ArrowOpt {
	prev: bool,
}

impl From<&Exec> for ArrowOpt {
	fn from(e: &Exec) -> Self { Self { prev: e.named.contains_key("previous") } }
}

impl Tab {
	pub fn find<'a>(&mut self, opt: impl Into<Opt<'a>>) -> bool {
		let opt = opt.into() as Opt;
		tokio::spawn(async move {
			let title = if opt.prev { "Find previous:" } else { "Find next:" };
			let rx = emit!(Input(
				InputOpt::from_cfg(title, &INPUTBOX.find_position, &INPUTBOX.find_offset).with_realtime()
			));

			let rx = Debounce::new(UnboundedReceiverStream::new(rx), Duration::from_millis(50));
			pin!(rx);

			while let Some(Ok(s)) | Some(Err(InputError::Typed(s))) = rx.next().await {
				emit!(Call(
					Exec::call("find_do", vec![s])
						.with_bool("previous", opt.prev)
						.with_bool("smart", opt.case == FinderCase::Smart)
						.with_bool("insensitive", opt.case == FinderCase::Insensitive)
						.vec(),
					KeymapLayer::Manager
				));
			}
		});
		false
	}

	pub fn find_do<'a>(&mut self, opt: impl Into<Opt<'a>>) -> bool {
		let opt = opt.into() as Opt;
		let Some(query) = opt.query else {
			return false;
		};

		let Ok(finder) = Finder::new(query, opt.case) else {
			return false;
		};

		let step = if opt.prev {
			finder.prev(&self.current.files, self.current.cursor, true)
		} else {
			finder.next(&self.current.files, self.current.cursor, true)
		};

		if let Some(step) = step {
			self.arrow(step);
		}

		self.finder = Some(finder);
		true
	}

	pub fn find_arrow(&mut self, opt: impl Into<ArrowOpt>) -> bool {
		let Some(finder) = &mut self.finder else {
			return false;
		};

		let b = finder.catchup(&self.current.files);
		let step = if opt.into().prev {
			finder.prev(&self.current.files, self.current.cursor, false)
		} else {
			finder.next(&self.current.files, self.current.cursor, false)
		};

		b | step.is_some_and(|s| self.arrow(s))
	}
}
