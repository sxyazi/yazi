use serde::Deserialize;
use yazi_shared::{MIME_DIR, event::Cmd, url::Url};

use crate::{Pattern, Priority};

#[derive(Debug, Deserialize)]
pub struct Fetcher {
	#[serde(skip)]
	pub idx: u8,

	pub id:   String,
	pub url:  Option<Pattern>,
	pub mime: Option<Pattern>,
	pub run:  Cmd,
	#[serde(default)]
	pub prio: Priority,
}

impl Fetcher {
	#[inline]
	pub fn matches(&self, url: &Url, mime: &str) -> bool {
		self.mime.as_ref().is_some_and(|p| p.match_mime(mime))
			|| self.url.as_ref().is_some_and(|p| p.match_url(url, mime == MIME_DIR))
	}
}
