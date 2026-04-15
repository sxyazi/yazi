use std::ops::Deref;

use serde::Deserialize;
use yazi_fs::File;
use yazi_shared::{Id, event::Action};

use crate::{Mixable, Pattern, Priority, plugin::preloader_id};

#[derive(Debug, Deserialize)]
pub struct Preloader {
	#[serde(skip, default = "preloader_id")]
	pub id:   Id,
	#[serde(skip)]
	pub idx:  u8,
	pub url:  Option<Pattern>,
	pub mime: Option<Pattern>,
	pub run:  Action,
	#[serde(default)]
	pub next: bool,
	#[serde(default)]
	pub prio: Priority,
}

impl Deref for Preloader {
	type Target = Action;

	fn deref(&self) -> &Self::Target { &self.run }
}

impl Preloader {
	#[inline]
	pub fn matches(&self, file: &File, mime: &str) -> bool { self.match_with(Some(file), Some(mime)) }

	pub fn match_with(&self, file: Option<&File>, mime: Option<&str>) -> bool {
		match (file, mime, &self.url, &self.mime) {
			(Some(f), _, Some(p), _) => p.match_url(&f.url, f.is_dir()),
			(_, Some(m), _, Some(p)) => p.match_mime(m),
			(None, None, ..) => true,
			_ => false,
		}
	}
}

impl Mixable for Preloader {}
