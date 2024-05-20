use std::path::Path;

use serde::Deserialize;
use yazi_shared::MIME_DIR;

use super::{Prefetcher, Preloader, Previewer};
use crate::{plugin::MAX_PREWORKERS, Preset, MERGED_YAZI};

#[derive(Deserialize)]
pub struct Plugin {
	pub prefetchers: Vec<Prefetcher>,
	pub preloaders:  Vec<Preloader>,
	pub previewers:  Vec<Previewer>,
}

impl Default for Plugin {
	fn default() -> Self {
		#[derive(Deserialize)]
		struct Outer {
			plugin: Shadow,
		}

		#[derive(Deserialize)]
		struct Shadow {
			prefetchers:         Vec<Prefetcher>,
			#[serde(default)]
			prepend_prefetchers: Vec<Prefetcher>,
			#[serde(default)]
			append_prefetchers:  Vec<Prefetcher>,

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

		let mut shadow = toml::from_str::<Outer>(&MERGED_YAZI).unwrap().plugin;
		if shadow.append_previewers.iter().any(|r| r.any_file()) {
			shadow.previewers.retain(|r| !r.any_file());
		}
		if shadow.append_previewers.iter().any(|r| r.any_dir()) {
			shadow.previewers.retain(|r| !r.any_dir());
		}

		Preset::mix(&mut shadow.prefetchers, shadow.prepend_prefetchers, shadow.append_prefetchers);
		Preset::mix(&mut shadow.preloaders, shadow.prepend_preloaders, shadow.append_preloaders);
		Preset::mix(&mut shadow.previewers, shadow.prepend_previewers, shadow.append_previewers);

		if shadow.prefetchers.len() + shadow.preloaders.len() > MAX_PREWORKERS as usize {
			panic!("Prefetchers and preloaders exceed the limit of {MAX_PREWORKERS}");
		}

		for (i, preloader) in shadow.prefetchers.iter_mut().enumerate() {
			preloader.id = i as u8;
		}
		for (i, preloader) in shadow.preloaders.iter_mut().enumerate() {
			preloader.id = shadow.prefetchers.len() as u8 + i as u8;
		}

		Self {
			prefetchers: shadow.prefetchers,
			preloaders:  shadow.preloaders,
			previewers:  shadow.previewers,
		}
	}
}

impl Plugin {
	pub fn prefetchers(
		&self,
		path: &Path,
		mime: Option<&str>,
		f: impl Fn(&str) -> bool + Copy,
	) -> Vec<&Prefetcher> {
		let is_dir = mime == Some(MIME_DIR);
		self
			.prefetchers
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
