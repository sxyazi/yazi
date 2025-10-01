use serde::Deserialize;
use yazi_fs::File;
use yazi_shared::event::Cmd;

use crate::Pattern;

#[derive(Debug, Deserialize)]
pub struct Spotter {
	pub url:  Option<Pattern>,
	pub mime: Option<Pattern>,
	pub run:  Cmd,
}

impl Spotter {
	#[inline]
	pub fn matches(&self, file: &File, mime: &str) -> bool {
		self.mime.as_ref().is_some_and(|p| p.match_mime(mime))
			|| self.url.as_ref().is_some_and(|p| p.match_url(&file.url, file.is_dir()))
	}

	#[inline]
	pub fn any_file(&self) -> bool { self.url.as_ref().is_some_and(|p| p.any_file()) }

	#[inline]
	pub fn any_dir(&self) -> bool { self.url.as_ref().is_some_and(|p| p.any_dir()) }
}
