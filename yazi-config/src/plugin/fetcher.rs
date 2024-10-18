use std::path::Path;

use serde::Deserialize;
use yazi_shared::{Condition, MIME_DIR, event::Cmd};

use crate::{Pattern, Priority};

#[derive(Debug, Deserialize)]
pub struct Fetcher {
	#[serde(skip)]
	pub idx: u8,

	pub id:   String,
	#[serde(rename = "if")]
	pub if_:  Option<Condition>,
	pub name: Option<Pattern>,
	pub mime: Option<Pattern>,
	pub run:  Cmd,
	#[serde(default)]
	pub prio: Priority,
}

impl Fetcher {
	#[inline]
	pub fn matches(&self, path: &Path, mime: &str, f: impl Fn(&str) -> bool + Copy) -> bool {
		self.if_.as_ref().and_then(|c| c.eval(f)) != Some(false)
			&& (self.mime.as_ref().is_some_and(|p| p.match_mime(mime))
				|| self.name.as_ref().is_some_and(|p| p.match_path(path, mime == MIME_DIR)))
	}
}

#[derive(Debug, Clone)]
pub struct FetcherProps {
	pub id:   u8,
	pub name: &'static str,
	pub prio: Priority,
}

impl From<&'static Fetcher> for FetcherProps {
	fn from(fetcher: &'static Fetcher) -> Self {
		Self { id: fetcher.idx, name: &fetcher.run.name, prio: fetcher.prio }
	}
}
