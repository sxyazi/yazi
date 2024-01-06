use std::path::Path;

use serde::Deserialize;
use yazi_shared::{event::Exec, Condition, MIME_DIR};

use crate::{pattern::Pattern, plugin::MAX_PRELOADERS, Priority, MERGED_YAZI};

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
	#[serde(default)]
	pub prio:  Priority,
}

impl Default for Plugin {
	fn default() -> Self {
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

		#[derive(Deserialize)]
		struct Outer {
			plugin: Shadow,
		}

		let mut shadow = toml::from_str::<Outer>(&MERGED_YAZI).unwrap().plugin;
		shadow.preloaders = shadow
			.prepend_preloaders
			.into_iter()
			.chain(shadow.preloaders)
			.chain(shadow.append_preloaders)
			.collect();
		shadow.previewers = shadow
			.prepend_previewers
			.into_iter()
			.chain(shadow.previewers)
			.chain(shadow.append_previewers)
			.collect();

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
