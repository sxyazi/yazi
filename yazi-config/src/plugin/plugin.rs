use anyhow::Result;
use serde::{Deserialize, de};
use yazi_codegen::DeserializeOver2;
use yazi_shim::toml::DeserializeOverHook;

use super::{Fetcher, Fetchers, Preloader, Preloaders, Previewer, Previewers, Spotter, Spotters};
use crate::{mix, plugin::{FetcherArc, PreloaderArc, PreviewerArc, SpotterArc}};

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
		let fetchers: Vec<FetcherArc> =
			mix(self.prepend_fetchers, self.fetchers.unwrap_unchecked(), self.append_fetchers);
		let spotters: Vec<SpotterArc> =
			mix(self.prepend_spotters, self.spotters.unwrap_unchecked(), self.append_spotters);
		let preloaders: Vec<PreloaderArc> =
			mix(self.prepend_preloaders, self.preloaders.unwrap_unchecked(), self.append_preloaders);
		let previewers: Vec<PreviewerArc> =
			mix(self.prepend_previewers, self.previewers.unwrap_unchecked(), self.append_previewers);

		Ok(Self {
			fetchers: fetchers.try_into().map_err(de::Error::custom)?,
			spotters: spotters.into(),
			preloaders: preloaders.try_into().map_err(de::Error::custom)?,
			previewers: previewers.into(),
			..Default::default()
		})
	}
}
