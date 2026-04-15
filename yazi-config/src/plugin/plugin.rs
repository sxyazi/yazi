use std::sync::Arc;

use anyhow::Result;
use serde::{Deserialize, de};
use yazi_codegen::DeserializeOver2;
use yazi_shim::toml::DeserializeOverHook;

use super::{Fetcher, Fetchers, Preloader, Preloaders, Previewer, Previewers, Spotter, Spotters};
use crate::{mix, plugin::{MAX_FETCHERS, MAX_PRELOADERS}};

#[derive(Default, Deserialize, DeserializeOver2)]
pub struct Plugin {
	pub fetchers:     Fetchers,
	#[serde(default)]
	prepend_fetchers: Vec<Fetcher>,
	#[serde(default)]
	append_fetchers:  Vec<Fetcher>,

	pub spotters:     Spotters,
	#[serde(default)]
	prepend_spotters: Vec<Spotter>,
	#[serde(default)]
	append_spotters:  Vec<Spotter>,

	pub preloaders:     Preloaders,
	#[serde(default)]
	prepend_preloaders: Vec<Preloader>,
	#[serde(default)]
	append_preloaders:  Vec<Preloader>,

	pub previewers:     Previewers,
	#[serde(default)]
	prepend_previewers: Vec<Previewer>,
	#[serde(default)]
	append_previewers:  Vec<Previewer>,
}

impl DeserializeOverHook for Plugin {
	fn deserialize_over_hook(self) -> Result<Self, toml::de::Error> {
		let mut fetchers: Vec<Arc<Fetcher>> =
			mix(self.prepend_fetchers, self.fetchers.unwrap_unchecked(), self.append_fetchers);
		let spotters: Vec<Arc<Spotter>> =
			mix(self.prepend_spotters, self.spotters.unwrap_unchecked(), self.append_spotters);
		let mut preloaders: Vec<Arc<Preloader>> =
			mix(self.prepend_preloaders, self.preloaders.unwrap_unchecked(), self.append_preloaders);
		let previewers: Vec<Arc<Previewer>> =
			mix(self.prepend_previewers, self.previewers.unwrap_unchecked(), self.append_previewers);

		if fetchers.len() > MAX_FETCHERS as usize {
			Err(de::Error::custom(format!("Fetchers exceed the limit of {MAX_FETCHERS}")))?;
		} else if preloaders.len() > MAX_PRELOADERS as usize {
			Err(de::Error::custom(format!("Preloaders exceed the limit of {MAX_PRELOADERS}")))?;
		}

		for (i, p) in fetchers.iter_mut().enumerate() {
			Arc::get_mut(p).ok_or_else(|| de::Error::custom("non-unique fetcher arc"))?.idx = i as u8;
		}
		for (i, p) in preloaders.iter_mut().enumerate() {
			Arc::get_mut(p).ok_or_else(|| de::Error::custom("non-unique preloader arc"))?.idx = i as u8;
		}

		Ok(Self {
			fetchers: fetchers.into(),
			spotters: spotters.into(),
			preloaders: preloaders.into(),
			previewers: previewers.into(),
			..Default::default()
		})
	}
}
