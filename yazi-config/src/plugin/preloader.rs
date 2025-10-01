use serde::Deserialize;
use yazi_fs::File;
use yazi_shared::event::Cmd;

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
	pub fn matches(&self, file: &File, mime: &str) -> bool {
		self.mime.as_ref().is_some_and(|p| p.match_mime(mime))
			|| self.url.as_ref().is_some_and(|p| p.match_url(&file.url, file.is_dir()))
	}
}
