use std::{collections::HashSet, path::Path};

use anyhow::Result;
use serde::Deserialize;
use tracing::warn;
use yazi_codegen::DeserializeOver2;
use yazi_fs::File;

use super::{Fetcher, Preloader, Previewer, Spotter};
use crate::{Preset, plugin::MAX_PREWORKERS};

#[derive(Default, Deserialize, DeserializeOver2)]
pub struct Plugin {
	pub fetchers:     Vec<Fetcher>,
	#[serde(default)]
	prepend_fetchers: Vec<Fetcher>,
	#[serde(default)]
	append_fetchers:  Vec<Fetcher>,

	pub spotters:     Vec<Spotter>,
	#[serde(default)]
	prepend_spotters: Vec<Spotter>,
	#[serde(default)]
	append_spotters:  Vec<Spotter>,

	pub preloaders:     Vec<Preloader>,
	#[serde(default)]
	prepend_preloaders: Vec<Preloader>,
	#[serde(default)]
	append_preloaders:  Vec<Preloader>,

	pub previewers:     Vec<Previewer>,
	#[serde(default)]
	prepend_previewers: Vec<Previewer>,
	#[serde(default)]
	append_previewers:  Vec<Previewer>,
}

impl Plugin {
	pub fn fetchers<'a, 'b: 'a>(
		&'b self,
		path: &'a Path,
		mime: &'a str,
	) -> impl Iterator<Item = &'b Fetcher> + 'a {
		let mut seen = HashSet::new();
		self.fetchers.iter().filter(move |&f| {
			if seen.contains(&f.id) || !f.matches(path, mime) {
				return false;
			}
			seen.insert(&f.id);
			true
		})
	}

	pub fn mime_fetchers(&self, files: Vec<File>) -> impl Iterator<Item = (&Fetcher, Vec<File>)> {
		let mut tasks: [Vec<_>; MAX_PREWORKERS as usize] = Default::default();
		for f in files {
			let found = self.fetchers.iter().find(|&g| g.id == "mime" && g.matches(&f.url, ""));
			if let Some(g) = found {
				tasks[g.idx as usize].push(f);
			} else {
				warn!("No mime fetcher for {f:?}");
			}
		}

		tasks.into_iter().enumerate().filter_map(|(i, tasks)| {
			if tasks.is_empty() { None } else { Some((&self.fetchers[i], tasks)) }
		})
	}

	pub fn spotter(&self, path: &Path, mime: &str) -> Option<&Spotter> {
		self.spotters.iter().find(|&p| p.matches(path, mime))
	}

	pub fn preloaders<'a, 'b: 'a>(
		&'b self,
		path: &'a Path,
		mime: &'a str,
	) -> impl Iterator<Item = &'b Preloader> + 'a {
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

impl Plugin {
	// TODO: remove .retain() and .collect()
	pub(crate) fn reshape(mut self) -> Result<Self> {
		if self.append_spotters.iter().any(|r| r.any_file()) {
			self.spotters.retain(|r| !r.any_file());
		}
		if self.append_spotters.iter().any(|r| r.any_dir()) {
			self.spotters.retain(|r| !r.any_dir());
		}
		if self.append_previewers.iter().any(|r| r.any_file()) {
			self.previewers.retain(|r| !r.any_file());
		}
		if self.append_previewers.iter().any(|r| r.any_dir()) {
			self.previewers.retain(|r| !r.any_dir());
		}

		self.fetchers =
			Preset::mix(self.prepend_fetchers, self.fetchers, self.append_fetchers).collect();
		self.spotters =
			Preset::mix(self.prepend_spotters, self.spotters, self.append_spotters).collect();
		self.preloaders =
			Preset::mix(self.prepend_preloaders, self.preloaders, self.append_preloaders).collect();
		self.previewers =
			Preset::mix(self.prepend_previewers, self.previewers, self.append_previewers).collect();

		if self.fetchers.len() + self.preloaders.len() > MAX_PREWORKERS as usize {
			panic!("Fetchers and preloaders exceed the limit of {MAX_PREWORKERS}");
		}

		for (i, p) in self.fetchers.iter_mut().enumerate() {
			p.idx = i as u8;
		}
		for (i, p) in self.preloaders.iter_mut().enumerate() {
			p.idx = self.fetchers.len() as u8 + i as u8;
		}

		Ok(Self {
			fetchers: self.fetchers,
			spotters: self.spotters,
			preloaders: self.preloaders,
			previewers: self.previewers,
			..Default::default()
		})
	}
}
