use std::path::Path;

use serde::Deserialize;
use yazi_shared::{event::Cmd, Condition, MIME_DIR};

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
	pub fn matches(&self, path: &Path, mime: Option<&str>, f: impl Fn(&str) -> bool + Copy) -> bool {
		self.if_.as_ref().and_then(|c| c.eval(f)) != Some(false)
			&& (self.mime.as_ref().zip(mime).map_or(false, |(p, m)| p.match_mime(m))
				|| self.name.as_ref().is_some_and(|p| p.match_path(path, mime == Some(MIME_DIR))))
	}
}

#[derive(Debug, Clone)]
pub struct FetcherProps {
	pub id:   u8,
	pub name: String,
	pub prio: Priority,
}

impl From<&Fetcher> for FetcherProps {
	fn from(fetcher: &Fetcher) -> Self {
		Self { id: fetcher.idx, name: fetcher.run.name.to_owned(), prio: fetcher.prio }
	}
}
