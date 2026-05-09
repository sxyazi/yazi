use anyhow::{Result, ensure};
use serde::{Deserialize, Deserializer, de};
use yazi_shim::toml::DeserializeOverWith;

use crate::{Mixable, Pattern, Selectable};

#[derive(Clone, Debug)]
pub struct Selector {
	pub url:  Option<Pattern>,
	pub mime: Option<Pattern>,
}

impl<'de> Deserialize<'de> for Selector {
	fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		#[derive(Deserialize)]
		struct Shadow {
			url:  Option<Pattern>,
			mime: Option<Pattern>,
		}

		let shadow = Shadow::deserialize(deserializer)?;
		Self::new(shadow.url, shadow.mime).map_err(de::Error::custom)
	}
}

impl DeserializeOverWith for Selector {
	fn deserialize_over_with<'de, D: Deserializer<'de>>(
		self,
		deserializer: D,
	) -> Result<Self, D::Error> {
		let new = Self::deserialize(deserializer)?;
		Self::new(new.url.or(self.url), new.mime.or(self.mime)).map_err(de::Error::custom)
	}
}

impl Selector {
	pub fn new(url: Option<Pattern>, mime: Option<Pattern>) -> Result<Self> {
		ensure!(url.is_some() || mime.is_some(), "at least one of `url` or `mime` must be specified");
		Ok(Self { url, mime })
	}
}

impl Selectable for Selector {
	fn url_pat(&self) -> Option<&Pattern> { self.url.as_ref() }

	fn mime_pat(&self) -> Option<&Pattern> { self.mime.as_ref() }
}

impl Mixable for Selector {
	fn any_file(&self) -> bool { self.url_pat().is_some_and(|p| p.any_file()) }

	fn any_dir(&self) -> bool { self.url_pat().is_some_and(|p| p.any_dir()) }
}
