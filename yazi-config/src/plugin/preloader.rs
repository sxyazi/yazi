use std::path::Path;

use serde::Deserialize;
use yazi_shared::{event::Cmd, MIME_DIR};

use crate::{Pattern, Priority};

#[derive(Debug, Deserialize)]
pub struct Preloader {
	#[serde(skip)]
	pub idx: u8,

	pub name: Option<Pattern>,
	pub mime: Option<Pattern>,
	pub run:  Cmd,
	#[serde(default)]
	pub next: bool,
	#[serde(default)]
	pub prio: Priority,
}

impl Preloader {
	#[inline]
	pub fn matches(&self, path: &Path, mime: Option<&str>) -> bool {
		self.mime.as_ref().zip(mime).map_or(false, |(p, m)| p.match_mime(m))
			|| self.name.as_ref().is_some_and(|p| p.match_path(path, mime == Some(MIME_DIR)))
	}
}

#[derive(Debug, Clone)]
pub struct PreloaderProps {
	pub id:   u8,
	pub name: String,
	pub prio: Priority,
}

impl From<&Preloader> for PreloaderProps {
	fn from(preloader: &Preloader) -> Self {
		Self { id: preloader.idx, name: preloader.run.name.to_owned(), prio: preloader.prio }
	}
}
