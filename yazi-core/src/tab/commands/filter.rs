use std::time::Duration;

use tokio::pin;
use tokio_stream::{wrappers::UnboundedReceiverStream, StreamExt};
use yazi_config::popup::InputCfg;
use yazi_fs::{Filter, FilterCase};
use yazi_proxy::{InputProxy, ManagerProxy};
use yazi_shared::{emit, event::Cmd, render, Debounce, InputError, Layer};

use crate::tab::Tab;

#[derive(Default)]
pub struct Opt {
	pub query: String,
	pub case:  FilterCase,
	pub done:  bool,
}

impl From<Cmd> for Opt {
	fn from(mut c: Cmd) -> Self {
		Self {
			query: c.take_first_str().unwrap_or_default(),
			case:  FilterCase::from(&c),
			done:  c.bool("done"),
		}
	}
}

impl Tab {
	pub fn filter(&mut self, opt: impl Into<Opt>) {
		let opt = opt.into() as Opt;
		tokio::spawn(async move {
			let rx = InputProxy::show(InputCfg::filter());

			let rx = Debounce::new(UnboundedReceiverStream::new(rx), Duration::from_millis(50));
			pin!(rx);

			while let Some(result) = rx.next().await {
				let done = result.is_ok();
				let (Ok(s) | Err(InputError::Typed(s))) = result else { continue };

				emit!(Call(
					Cmd::args("filter_do", &[s])
						.with_bool("smart", opt.case == FilterCase::Smart)
						.with_bool("insensitive", opt.case == FilterCase::Insensitive)
						.with_bool("done", done),
					Layer::Manager
				));
			}
		});
	}

	pub fn filter_do(&mut self, opt: impl Into<Opt>) {
		let opt = opt.into() as Opt;

		let filter = if opt.query.is_empty() {
			None
		} else if let Ok(f) = Filter::new(&opt.query, opt.case) {
			Some(f)
		} else {
			return;
		};

		if opt.done {
			ManagerProxy::update_paged(); // Update for paged files in next loop
		}

		let hovered = self.current.hovered().map(|f| f.url());
		if !self.current.files.set_filter(filter) {
			return;
		}

		self.current.repos(hovered.as_ref());
		if self.current.hovered().map(|f| &f.url) != hovered.as_ref() {
			ManagerProxy::hover(None, self.idx);
		}

		render!();
	}
}
