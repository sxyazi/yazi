use std::time::Duration;

use tokio::pin;
use tokio_stream::{wrappers::UnboundedReceiverStream, StreamExt};
use yazi_config::popup::InputCfg;
use yazi_shared::{emit, event::Exec, Debounce, InputError, Layer};

use crate::{input::Input, tab::Tab};

pub struct Opt<'a> {
	query: Option<&'a str>,
}

impl<'a> From<&'a Exec> for Opt<'a> {
	fn from(e: &'a Exec) -> Self { Self { query: e.args.first().map(|s| s.as_str()) } }
}

impl Tab {
	pub fn filter<'a>(&mut self, _opt: impl Into<Opt<'a>>) -> bool {
		tokio::spawn(async move {
			let rx = Input::_show(InputCfg::filter());

			let rx = Debounce::new(UnboundedReceiverStream::new(rx), Duration::from_millis(50));
			pin!(rx);

			while let Some(Ok(s)) | Some(Err(InputError::Typed(s))) = rx.next().await {
				emit!(Call(Exec::call("filter_do", vec![s]).vec(), Layer::Manager));
			}
		});
		false
	}

	pub fn filter_do<'a>(&mut self, opt: impl Into<Opt<'a>>) -> bool {
		let opt = opt.into() as Opt;
		let Some(query) = opt.query else {
			return false;
		};

		let hovered = &self.current.hovered().map(|f| f.url());

		self.current.files.set_filter_keyword(query);

		if let Some(hovered_url) = hovered {
			match (self.current.files.position(hovered_url), self.current.files.first()) {
				(Some(_), _) => {
					self.current.hover(hovered_url);
				}
				(None, Some(first)) => {
					self.current.hover(&first.url());
				}
				(..) => {}
			}
		}

		true
	}
}
