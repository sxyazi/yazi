use std::{path::Path, str::FromStr};

use serde::Deserialize;
use yazi_shared::MIME_DIR;

use super::{Fetcher, Preloader, Previewer};
use crate::{plugin::MAX_PREWORKERS, Preset};

#[derive(Deserialize)]
pub struct Plugin {
	pub fetchers:   Vec<Fetcher>,
	pub preloaders: Vec<Preloader>,
	pub previewers: Vec<Previewer>,
}

impl Plugin {
	pub fn fetchers(
		&self,
		path: &Path,
		mime: Option<&str>,
		f: impl Fn(&str) -> bool + Copy,
	) -> Vec<&Fetcher> {
		let is_dir = mime == Some(MIME_DIR);
		self
			.fetchers
			.iter()
			.filter(|&p| {
				p.cond.as_ref().and_then(|c| c.eval(f)) != Some(false)
					&& (p.mime.as_ref().zip(mime).map_or(false, |(p, m)| p.match_mime(m))
						|| p.name.as_ref().is_some_and(|p| p.match_path(path, is_dir)))
			})
			.collect()
	}

	pub fn preloaders(&self, path: &Path, mime: Option<&str>) -> Vec<&Preloader> {
		let is_dir = mime == Some(MIME_DIR);
		let mut preloaders = Vec::with_capacity(1);

		for p in &self.preloaders {
			if !p.mime.as_ref().zip(mime).map_or(false, |(p, m)| p.match_mime(m))
				&& !p.name.as_ref().is_some_and(|p| p.match_path(path, is_dir))
			{
				continue;
			}

			preloaders.push(p);
			if !p.next {
				break;
			}
		}
		preloaders
	}

	pub fn previewer(&self, path: &Path, mime: &str) -> Option<&Previewer> {
		let is_dir = mime == MIME_DIR;
		self.previewers.iter().find(|&p| {
			p.mime.as_ref().is_some_and(|p| p.match_mime(mime))
				|| p.name.as_ref().is_some_and(|p| p.match_path(path, is_dir))
		})
	}
}
impl FromStr for Plugin {
	type Err = toml::de::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		#[derive(Deserialize)]
		struct Outer {
			plugin: Shadow,
		}

		#[derive(Deserialize)]
		struct Shadow {
			fetchers:         Vec<Fetcher>,
			#[serde(default)]
			prepend_fetchers: Vec<Fetcher>,
			#[serde(default)]
			append_fetchers:  Vec<Fetcher>,

			preloaders:         Vec<Preloader>,
			#[serde(default)]
			prepend_preloaders: Vec<Preloader>,
			#[serde(default)]
			append_preloaders:  Vec<Preloader>,

			previewers:         Vec<Previewer>,
			#[serde(default)]
			prepend_previewers: Vec<Previewer>,
			#[serde(default)]
			append_previewers:  Vec<Previewer>,
		}

		let mut shadow = toml::from_str::<Outer>(s)?.plugin;
		if shadow.append_previewers.iter().any(|r| r.any_file()) {
			shadow.previewers.retain(|r| !r.any_file());
		}
		if shadow.append_previewers.iter().any(|r| r.any_dir()) {
			shadow.previewers.retain(|r| !r.any_dir());
		}

		Preset::mix(&mut shadow.fetchers, shadow.prepend_fetchers, shadow.append_fetchers);
		Preset::mix(&mut shadow.preloaders, shadow.prepend_preloaders, shadow.append_preloaders);
		Preset::mix(&mut shadow.previewers, shadow.prepend_previewers, shadow.append_previewers);

		if shadow.fetchers.len() + shadow.preloaders.len() > MAX_PREWORKERS as usize {
			panic!("Fetchers and preloaders exceed the limit of {MAX_PREWORKERS}");
		}

		for (i, p) in shadow.fetchers.iter_mut().enumerate() {
			p.idx = i as u8;
		}
		for (i, p) in shadow.preloaders.iter_mut().enumerate() {
			p.idx = shadow.fetchers.len() as u8 + i as u8;
		}

		Ok(Self {
			fetchers:   shadow.fetchers,
			preloaders: shadow.preloaders,
			previewers: shadow.previewers,
		})
	}
}
