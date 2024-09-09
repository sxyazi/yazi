use std::borrow::Cow;

use yazi_proxy::AppProxy;
use yazi_shared::{event::{Cmd, Data}, fs::Url, render, render_and};

use crate::tab::Tab;

pub struct Opt<'a> {
	url:   Option<Cow<'a, Url>>,
	state: Option<bool>,
}

impl<'a> From<Cmd> for Opt<'a> {
	fn from(mut c: Cmd) -> Self {
		Self {
			url:   c.take("url").and_then(Data::into_url).map(Cow::Owned),
			state: match c.take_str("state").as_deref() {
				Some("true") => Some(true),
				Some("false") => Some(false),
				_ => None,
			},
		}
	}
}

impl<'a> Tab {
	pub fn select(&mut self, opt: impl Into<Opt<'a>>) {
		let opt = opt.into() as Opt;
		let Some(url) = opt.url.or_else(|| self.current.hovered().map(|h| Cow::Borrowed(h.url())))
		else {
			return;
		};

		let b = match opt.state {
			Some(true) => render_and!(self.selected.add(&url)),
			Some(false) => render_and!(self.selected.remove(&url)) | true,
			None => render_and!(self.selected.remove(&url) || self.selected.add(&url)),
		};

		if !b {
			AppProxy::notify_warn(
				"Select one",
				"This file cannot be selected, due to path nesting conflict.",
			);
		}
	}
}
