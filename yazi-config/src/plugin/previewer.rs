use std::path::Path;

use serde::Deserialize;
use yazi_shared::{MIME_DIR, event::Cmd};

use crate::Pattern;

#[derive(Debug, Deserialize)]
pub struct Previewer {
	pub name: Option<Pattern>,
	pub mime: Option<Pattern>,
	pub run:  Cmd,
}

impl Previewer {
	#[inline]
	pub fn matches(&self, path: &Path, mime: &str) -> bool {
		self.mime.as_ref().is_some_and(|p| p.match_mime(mime))
			|| self.name.as_ref().is_some_and(|p| p.match_path(path, mime == MIME_DIR))
	}

	#[inline]
	pub fn any_file(&self) -> bool { self.name.as_ref().is_some_and(|p| p.any_file()) }

	#[inline]
	pub fn any_dir(&self) -> bool { self.name.as_ref().is_some_and(|p| p.any_dir()) }
}
