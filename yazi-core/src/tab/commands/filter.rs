use std::time::Duration;

use tokio::pin;
use tokio_stream::{wrappers::UnboundedReceiverStream, StreamExt};
use yazi_config::popup::InputCfg;
use yazi_shared::{emit, event::Exec, Debounce, InputError, Layer};

use crate::{folder::{Filter, FilterCase}, input::Input, manager::Manager, tab::Tab};

#[derive(Default)]
pub struct Opt<'a> {
	pub query: Option<&'a str>,
	pub case:  FilterCase,
}

impl<'a> From<&'a Exec> for Opt<'a> {
	fn from(e: &'a Exec) -> Self {
		Self { query: e.args.first().map(|s| s.as_str()), case: e.into() }
	}
}

impl Tab {
	pub fn filter<'a>(&mut self, opt: impl Into<Opt<'a>>) -> bool {
		let opt = opt.into() as Opt;
		tokio::spawn(async move {
			let rx = Input::_show(InputCfg::filter());

			let rx = Debounce::new(UnboundedReceiverStream::new(rx), Duration::from_millis(50));
			pin!(rx);

			while let Some(Ok(s)) | Some(Err(InputError::Typed(s))) = rx.next().await {
				emit!(Call(
					Exec::call("filter_do", vec![s])
						.with_bool("smart", opt.case == FilterCase::Smart)
						.with_bool("insensitive", opt.case == FilterCase::Insensitive)
						.vec(),
					Layer::Manager
				));
			}
		});
		false
	}

	pub fn filter_do<'a>(&mut self, opt: impl Into<Opt<'a>>) -> bool {
		let opt = opt.into() as Opt;
		let Some(query) = opt.query else {
			return false;
		};

		let filter = if query.is_empty() {
			None
		} else if let Ok(f) = Filter::new(query, opt.case) {
			Some(f)
		} else {
			return false;
		};

		let hovered = self.current.hovered().map(|f| f.url());
		if !self.current.files.set_filter(filter) {
			return false;
		}

		if self.current.repos(hovered) {
			Manager::_hover(None);
		}
		true
	}
}
