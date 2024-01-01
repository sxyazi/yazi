use std::time::Duration;

use tokio::pin;
use tokio_stream::{wrappers::UnboundedReceiverStream, StreamExt};
use yazi_config::popup::InputCfg;
use yazi_shared::{emit, event::Exec, render, Debounce, InputError, Layer};

use crate::{folder::FilterCase, input::Input, tab::{Finder, Tab}};

pub struct Opt<'a> {
	query: Option<&'a str>,
	prev:  bool,
	case:  FilterCase,
}

impl<'a> From<&'a Exec> for Opt<'a> {
	fn from(e: &'a Exec) -> Self {
		Self {
			query: e.args.first().map(|s| s.as_str()),
			prev:  e.named.contains_key("previous"),
			case:  e.into(),
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
	pub fn find<'a>(&mut self, opt: impl Into<Opt<'a>>) {
		let opt = opt.into() as Opt;
		tokio::spawn(async move {
			let rx = Input::_show(InputCfg::find(opt.prev));

			let rx = Debounce::new(UnboundedReceiverStream::new(rx), Duration::from_millis(50));
			pin!(rx);

			while let Some(Ok(s)) | Some(Err(InputError::Typed(s))) = rx.next().await {
				emit!(Call(
					Exec::call("find_do", vec![s])
						.with_bool("previous", opt.prev)
						.with_bool("smart", opt.case == FilterCase::Smart)
						.with_bool("insensitive", opt.case == FilterCase::Insensitive)
						.vec(),
					Layer::Manager
				));
			}
		});
	}

	pub fn find_do<'a>(&mut self, opt: impl Into<Opt<'a>>) {
		let opt = opt.into() as Opt;
		let Some(query) = opt.query else {
			return;
		};
		if query.is_empty() {
			return self.escape(super::escape::Opt::FIND);
		}

		let Ok(finder) = Finder::new(query, opt.case) else {
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
