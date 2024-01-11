use std::time::Duration;

use tokio::pin;
use tokio_stream::{wrappers::UnboundedReceiverStream, StreamExt};
use yazi_config::popup::InputCfg;
use yazi_shared::{emit, event::Exec, render, Debounce, InputError, Layer};

use crate::{folder::{Filter, FilterCase}, input::Input, manager::Manager, tab::Tab};

#[derive(Default)]
pub struct Opt<'a> {
	pub query: &'a str,
	pub case:  FilterCase,
	pub done:  bool,
}

impl<'a> From<&'a Exec> for Opt<'a> {
	fn from(e: &'a Exec) -> Self {
		Self {
			query: e.args.first().map(|s| s.as_str()).unwrap_or_default(),
			case:  e.into(),
			done:  e.named.contains_key("done"),
		}
	}
}

impl Tab {
	pub fn filter<'a>(&mut self, opt: impl Into<Opt<'a>>) {
		let opt = opt.into() as Opt;
		tokio::spawn(async move {
			let rx = Input::_show(InputCfg::filter());

			let rx = Debounce::new(UnboundedReceiverStream::new(rx), Duration::from_millis(50));
			pin!(rx);

			while let Some(result) = rx.next().await {
				let done = result.is_ok();
				let (Ok(s) | Err(InputError::Typed(s))) = result else {
					continue;
				};

				emit!(Call(
					Exec::call("filter_do", vec![s])
						.with_bool("smart", opt.case == FilterCase::Smart)
						.with_bool("insensitive", opt.case == FilterCase::Insensitive)
						.with_bool("done", done)
						.vec(),
					Layer::Manager
				));
			}
		});
	}

	pub fn filter_do<'a>(&mut self, opt: impl Into<Opt<'a>>) {
		let opt = opt.into() as Opt;

		let filter = if opt.query.is_empty() {
			None
		} else if let Ok(f) = Filter::new(opt.query, opt.case) {
			Some(f)
		} else {
			return;
		};

		if opt.done {
			Manager::_update_paged(); // Update for paged files in next loop
		}

		let hovered = self.current.hovered().map(|f| f.url());
		if !self.current.files.set_filter(filter) {
			return;
		}

		if self.current.repos(hovered) {
			Manager::_hover(None);
		}
		render!();
	}
}
