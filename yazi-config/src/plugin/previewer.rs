use serde::{Deserialize, Deserializer, de};
use yazi_shared::{MIME_DIR, event::Cmd, url::UrlBuf};

use crate::Pattern;

#[derive(Debug, Deserialize)]
pub struct Previewer {
	pub url:  Option<Pattern>,
	pub mime: Option<Pattern>,
	#[serde(deserialize_with = "deserialize_run")]
	pub run:  Vec<Cmd>,
}

impl Previewer {
	#[inline]
	pub fn matches(&self, url: &UrlBuf, mime: &str) -> bool {
		self.mime.as_ref().is_some_and(|p| p.match_mime(mime))
			|| self.url.as_ref().is_some_and(|p| p.match_url(url, mime == MIME_DIR))
	}

	#[inline]
	pub fn len(&self) -> usize { self.run.len() }

	#[inline]
	pub fn cmd(&self, index: usize) -> Option<&Cmd> {
		if self.run.is_empty() {
			return None;
		}
		self.run.get(index).or_else(|| self.run.first())
	}

	#[inline]
	pub fn any_file(&self) -> bool { self.url.as_ref().is_some_and(|p| p.any_file()) }

	#[inline]
	pub fn any_dir(&self) -> bool { self.url.as_ref().is_some_and(|p| p.any_dir()) }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum RunField {
	One(String),
	Many(Vec<String>),
}

fn deserialize_run<'de, D>(deserializer: D) -> Result<Vec<Cmd>, D::Error>
where
	D: Deserializer<'de>,
{
	let field = RunField::deserialize(deserializer)?;
	let items: Vec<String> = match field {
		RunField::One(s) => vec![s],
		RunField::Many(list) => list,
	};

	if items.is_empty() {
		return Err(de::Error::custom("previewer.run must not be empty"));
	}

	items
		.into_iter()
		.map(|s| s.parse::<Cmd>().map_err(de::Error::custom))
		.collect()
}
