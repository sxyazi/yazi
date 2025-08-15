use serde::Deserialize;
use yazi_shared::{MIME_DIR, event::Cmd, url::UrlBuf};

use crate::Pattern;

#[derive(Debug, Deserialize)]
pub struct Previewer {
	pub url:  Option<Pattern>,
	pub mime: Option<Pattern>,
	pub run:  Cmd,
}

impl Previewer {
	#[inline]
	pub fn matches(&self, url: &UrlBuf, mime: &str) -> bool {
		self.mime.as_ref().is_some_and(|p| p.match_mime(mime))
			|| self.url.as_ref().is_some_and(|p| p.match_url(url, mime == MIME_DIR))
	}

	#[inline]
	pub fn any_file(&self) -> bool { self.url.as_ref().is_some_and(|p| p.any_file()) }

	#[inline]
	pub fn any_dir(&self) -> bool { self.url.as_ref().is_some_and(|p| p.any_dir()) }
}
