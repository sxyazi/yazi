use std::time::Duration;

use tokio::pin;
use tokio_stream::{wrappers::UnboundedReceiverStream, StreamExt};
use yazi_config::popup::InputCfg;
use yazi_fs::FilterCase;
use yazi_proxy::InputProxy;
use yazi_shared::{emit, event::Cmd, render, Debounce, InputError, Layer};

use crate::tab::{Finder, Tab};

pub struct Opt {
	query: Option<String>,
	prev:  bool,
	case:  FilterCase,
}

impl From<Cmd> for Opt {
	fn from(mut c: Cmd) -> Self {
		Self { query: c.take_first_str(), prev: c.bool("previous"), case: FilterCase::from(&c) }
	}
}

pub struct ArrowOpt {
	prev: bool,
}

impl From<Cmd> for ArrowOpt {
	fn from(c: Cmd) -> Self { Self { prev: c.bool("previous") } }
}

impl Tab {
	pub fn find(&mut self, opt: impl Into<Opt>) {
		let opt = opt.into() as Opt;
		tokio::spawn(async move {
			let rx = InputProxy::show(InputCfg::find(opt.prev));

			let rx = Debounce::new(UnboundedReceiverStream::new(rx), Duration::from_millis(50));
			pin!(rx);

			while let Some(Ok(s)) | Some(Err(InputError::Typed(s))) = rx.next().await {
				emit!(Call(
					Cmd::args("find_do", &[s])
						.with_bool("previous", opt.prev)
						.with_bool("smart", opt.case == FilterCase::Smart)
						.with_bool("insensitive", opt.case == FilterCase::Insensitive),
					Layer::Manager
				));
			}
		});
	}

	pub fn find_do(&mut self, opt: impl Into<Opt>) {
		let opt = opt.into() as Opt;
		let Some(query) = opt.query else {
			return;
		};
		if query.is_empty() {
			self.escape_find();
			return;
		}

		let Ok(finder) = Finder::new(&query, opt.case) else {
			return;
		};
		if matches!(&self.finder, Some(f) if f.filter == finder.filter) {
			return;
		}

		let step = if opt.prev {
			finder.prev(&self.current.files, self.current.cursor, true)
		} else {
			finder.next(&self.current.files, self.current.cursor, true)
		};

		if let Some(step) = step {
			self.arrow(step);
		}

		self.finder = Some(finder);
		render!();
	}

	pub fn find_arrow(&mut self, opt: impl Into<ArrowOpt>) {
		let Some(finder) = &mut self.finder else {
			return;
		};

		render!(finder.catchup(&self.current.files));
		if opt.into().prev {
			finder.prev(&self.current.files, self.current.cursor, false).map(|s| self.arrow(s));
		} else {
			finder.next(&self.current.files, self.current.cursor, false).map(|s| self.arrow(s));
		}
	}
}
