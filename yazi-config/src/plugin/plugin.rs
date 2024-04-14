use std::path::Path;

use serde::Deserialize;
use yazi_shared::MIME_DIR;

use super::PluginRule;
use crate::{plugin::MAX_PRELOADERS, Preset, MERGED_YAZI};

#[derive(Deserialize)]
pub struct Plugin {
	pub preloaders: Vec<PluginRule>,
	pub previewers: Vec<PluginRule>,
}

impl Default for Plugin {
	fn default() -> Self {
		#[derive(Deserialize)]
		struct Outer {
			plugin: Shadow,
		}

		#[derive(Deserialize)]
		struct Shadow {
			preloaders:         Vec<PluginRule>,
			#[serde(default)]
			prepend_preloaders: Vec<PluginRule>,
			#[serde(default)]
			append_preloaders:  Vec<PluginRule>,

			previewers:         Vec<PluginRule>,
			#[serde(default)]
			prepend_previewers: Vec<PluginRule>,
			#[serde(default)]
			append_previewers:  Vec<PluginRule>,
		}

		let mut shadow = toml::from_str::<Outer>(&MERGED_YAZI).unwrap().plugin;
		if shadow.append_previewers.iter().any(|r| r.any_file()) {
			shadow.previewers.retain(|r| !r.any_file());
		}
		if shadow.append_previewers.iter().any(|r| r.any_dir()) {
			shadow.previewers.retain(|r| !r.any_dir());
		}

		Preset::mix(&mut shadow.preloaders, shadow.prepend_preloaders, shadow.append_preloaders);
		Preset::mix(&mut shadow.previewers, shadow.prepend_previewers, shadow.append_previewers);

		if shadow.preloaders.len() > MAX_PRELOADERS as usize {
			panic!("Too many preloaders");
		}

		for (i, preloader) in shadow.preloaders.iter_mut().enumerate() {
			if preloader.sync {
				panic!("Preloaders cannot be synchronous");
			}
			preloader.id = i as u8;
		}

		Self { preloaders: shadow.preloaders, previewers: shadow.previewers }
	}
}

impl Plugin {
	pub fn preloaders(
		&self,
		path: &Path,
		mime: Option<&str>,
		f: impl Fn(&str) -> bool + Copy,
	) -> Vec<&PluginRule> {
		let is_folder = mime == Some(MIME_DIR);
		self
			.preloaders
			.iter()
			.filter(|&rule| {
				rule.cond.as_ref().and_then(|c| c.eval(f)) != Some(false)
					&& (rule.mime.as_ref().zip(mime).map_or(false, |(p, m)| p.match_mime(m))
						|| rule.name.as_ref().is_some_and(|p| p.match_path(path, is_folder)))
			})
			.collect()
	}

	pub fn previewer(&self, path: &Path, mime: &str) -> Option<&PluginRule> {
		let is_folder = mime == MIME_DIR;
		self.previewers.iter().find(|&rule| {
			rule.mime.as_ref().is_some_and(|p| p.match_mime(mime))
				|| rule.name.as_ref().is_some_and(|p| p.match_path(path, is_folder))
		})
	}
}
