use std::{collections::HashSet, path::Path, str::FromStr};

use serde::Deserialize;

use super::{Fetcher, Preloader, Previewer};
use crate::{plugin::MAX_PREWORKERS, Preset};

#[derive(Deserialize)]
pub struct Plugin {
	pub fetchers:   Vec<Fetcher>,
	pub preloaders: Vec<Preloader>,
	pub previewers: Vec<Previewer>,
}

impl Plugin {
	pub fn fetchers<'a>(
		&'a self,
		path: &'a Path,
		mime: &'a str,
		factor: impl Fn(&str) -> bool + Copy,
	) -> impl Iterator<Item = &'a Fetcher> {
		let mut seen = HashSet::new();
		self.fetchers.iter().filter(move |&f| {
			if seen.contains(&f.id) || !f.matches(path, mime, factor) {
				return false;
			}
			seen.insert(&f.id);
			true
		})
	}

	#[inline]
	pub fn fetchers_mask(&self) -> u32 {
		self.fetchers.iter().fold(0, |n, f| if f.mime.is_some() { n } else { n | 1 << f.idx as u32 })
	}

	pub fn preloaders<'a>(
		&'a self,
		path: &'a Path,
		mime: &'a str,
	) -> impl Iterator<Item = &'a Preloader> {
		let mut next = true;
		self.preloaders.iter().filter(move |&p| {
			if !next || !p.matches(path, mime) {
				return false;
			}
			next = p.next;
			true
		})
	}

	pub fn previewer(&self, path: &Path, mime: &str) -> Option<&Previewer> {
		self.previewers.iter().find(|&p| p.matches(path, mime))
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

		shadow.fetchers =
			Preset::mix(shadow.fetchers, shadow.prepend_fetchers, shadow.append_fetchers).collect();
		shadow.preloaders =
			Preset::mix(shadow.preloaders, shadow.prepend_preloaders, shadow.append_preloaders).collect();
		shadow.previewers =
			Preset::mix(shadow.previewers, shadow.prepend_previewers, shadow.append_previewers).collect();

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
