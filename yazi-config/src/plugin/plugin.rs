use std::path::Path;

use serde::Deserialize;
use yazi_shared::{event::Exec, Condition, MIME_DIR};

use crate::{pattern::Pattern, plugin::MAX_PRELOADERS, MERGED_YAZI};

#[derive(Deserialize)]
pub struct Plugin {
	pub preloaders: Vec<PluginRule>,
	pub previewers: Vec<PluginRule>,
}

#[derive(Deserialize)]
pub struct PluginRule {
	#[serde(default)]
	pub id:    u8,
	pub cond:  Option<Condition>,
	pub name:  Option<Pattern>,
	pub mime:  Option<Pattern>,
	#[serde(deserialize_with = "super::exec_deserialize")]
	pub exec:  Exec,
	#[serde(default)]
	pub sync:  bool,
	#[serde(default)]
	pub multi: bool,
}

impl Default for Plugin {
	fn default() -> Self {
		#[derive(Deserialize)]
		struct Outer {
			plugin: Plugin,
		}

		let mut plugin = toml::from_str::<Outer>(&MERGED_YAZI).unwrap().plugin;
		if plugin.preloaders.len() > MAX_PRELOADERS as usize {
			panic!("Too many preloaders");
		}

		for (i, preloader) in plugin.preloaders.iter_mut().enumerate() {
			if preloader.sync {
				panic!("Preloaders cannot be synchronous");
			}
			preloader.id = i as u8;
		}

		plugin
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
					&& (rule.name.as_ref().is_some_and(|n| n.match_path(path, is_folder))
						|| rule.mime.as_ref().zip(mime).map_or(false, |(m, s)| m.matches(s)))
			})
			.collect()
	}

	pub fn previewer(&self, path: &Path, mime: &str) -> Option<&PluginRule> {
		let is_folder = mime == MIME_DIR;
		self.previewers.iter().find(|&rule| {
			rule.mime.as_ref().is_some_and(|m| m.matches(mime))
				|| rule.name.as_ref().is_some_and(|n| n.match_path(path, is_folder))
		})
	}
}
