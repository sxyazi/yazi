use std::ops::Deref;

use serde::Deserialize;
use yazi_fs::File;
use yazi_shared::{Id, event::Action};

use crate::{Mixable, Pattern, plugin::previewer_id};

#[derive(Debug, Deserialize)]
pub struct Previewer {
	#[serde(skip, default = "previewer_id")]
	pub id:   Id,
	pub url:  Option<Pattern>,
	pub mime: Option<Pattern>,
	pub run:  Action,
}

impl Deref for Previewer {
	type Target = Action;

	#[inline]
	fn deref(&self) -> &Self::Target { &self.run }
}

impl Previewer {
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

impl Mixable for Previewer {
	fn any_file(&self) -> bool { self.url.as_ref().is_some_and(|p| p.any_file()) }

	fn any_dir(&self) -> bool { self.url.as_ref().is_some_and(|p| p.any_dir()) }
}
