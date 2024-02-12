use std::borrow::Cow;

use yazi_shared::{event::Cmd, fs::Url, render};

use crate::tab::Tab;

pub struct Opt<'a> {
	url:   Option<Cow<'a, Url>>,
	state: Option<bool>,
}

impl<'a> From<Cmd> for Opt<'a> {
	fn from(mut c: Cmd) -> Self {
		Self {
			url:   c.take_name("url").map(|s| Cow::Owned(Url::from(s))),
			state: match c.named.get("state").map(|s| s.as_str()) {
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
		let Some(url) = opt.url.or_else(|| self.current.hovered().map(|h| Cow::Borrowed(&h.url)))
		else {
			return;
		};

		render!(match opt.state {
			Some(true) => self.selected.insert(url.into_owned()),
			Some(false) => self.selected.remove(&url),
			None => self.selected.remove(&url) || self.selected.insert(url.into_owned()),
		});
	}
}
