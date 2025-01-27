use std::path::Path;

use serde::Deserialize;
use yazi_shared::{MIME_DIR, event::Cmd};

use crate::{Pattern, Priority};

#[derive(Debug, Deserialize)]
pub struct Fetcher {
	#[serde(skip)]
	pub idx: u8,

	pub id:   String,
	pub name: Option<Pattern>,
	pub mime: Option<Pattern>,
	pub run:  Cmd,
	#[serde(default)]
	pub prio: Priority,
}

impl Fetcher {
	#[inline]
	pub fn matches(&self, path: &Path, mime: &str) -> bool {
		self.mime.as_ref().is_some_and(|p| p.match_mime(mime))
			|| self.name.as_ref().is_some_and(|p| p.match_path(path, mime == MIME_DIR))
	}
}
