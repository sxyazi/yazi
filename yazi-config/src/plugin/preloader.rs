use serde::Deserialize;
use yazi_shared::{MIME_DIR, event::Cmd, url::UrlBuf};

use crate::{Pattern, Priority};

#[derive(Debug, Deserialize)]
pub struct Preloader {
	#[serde(skip)]
	pub idx: u8,

	pub url:  Option<Pattern>,
	pub mime: Option<Pattern>,
	pub run:  Cmd,
	#[serde(default)]
	pub next: bool,
	#[serde(default)]
	pub prio: Priority,
}

impl Preloader {
	#[inline]
	pub fn matches(&self, url: &UrlBuf, mime: &str) -> bool {
		self.mime.as_ref().is_some_and(|p| p.match_mime(mime))
			|| self.url.as_ref().is_some_and(|p| p.match_url(url, mime == MIME_DIR))
	}
}
