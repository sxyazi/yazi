use yazi_dds::Pubsub;
use yazi_macro::{err, render};
use yazi_shared::{event::CmdCow, url::{Url, Urn}};

use crate::tab::Tab;

struct Opt {
	url: Option<Url>,
}

impl From<CmdCow> for Opt {
	fn from(mut c: CmdCow) -> Self { Self { url: c.take_first_url() } }
}
impl From<Option<Url>> for Opt {
	fn from(url: Option<Url>) -> Self { Self { url } }
}

impl Tab {
	#[yazi_codegen::command]
	pub fn hover(&mut self, opt: Opt) {
		if let Some(u) = opt.url {
			self.hover_do(u);
		} else {
			self.current.arrow(0);
		}

		// Publish through DDS
		err!(Pubsub::pub_from_hover(self.id, self.hovered().map(|h| &h.url)));
	}

	fn hover_do(&mut self, url: Url) {
		// Hover on the file
		if let Ok(p) = url.strip_prefix(self.cwd()) {
			render!(self.current.hover(Urn::new(p)));
		}

		// Turn on tracing
		if self.hovered().is_some_and(|h| h.url == url) {
			// `hover(Some)` occurs after user actions, such as create, rename, reveal, etc.
			// At this point, it's intuitive to track the location of the file regardless.
			self.current.trace = Some(url.urn_owned());
		}
	}
}
