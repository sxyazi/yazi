use std::path::Path;

use serde::Deserialize;
use yazi_shared::{event::Cmd, Condition, MIME_DIR};

use crate::{pattern::Pattern, plugin::MAX_PRELOADERS, Preset, Priority, MERGED_YAZI};

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
		if shadow.append_preloaders.iter().any(|r| r.any_file()) {
			shadow.preloaders.retain(|r| !r.any_file());
		}
		if shadow.append_preloaders.iter().any(|r| r.any_dir()) {
			shadow.preloaders.retain(|r| !r.any_dir());
		}
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

#[derive(Deserialize)]
pub struct PluginRule {
	#[serde(default)]
	pub id:    u8,
	pub cond:  Option<Condition>,
	pub name:  Option<Pattern>,
	pub mime:  Option<Pattern>,
	#[serde(rename = "exec")]
	#[serde(deserialize_with = "super::exec_deserialize")]
	pub cmd:   Cmd,
	#[serde(default)]
	pub sync:  bool,
	#[serde(default)]
	pub multi: bool,
	#[serde(default)]
	pub prio:  Priority,
}

impl PluginRule {
	#[inline]
	fn any_file(&self) -> bool { self.name.as_ref().is_some_and(|p| p.any_file()) }

	#[inline]
	fn any_dir(&self) -> bool { self.name.as_ref().is_some_and(|p| p.any_dir()) }
}
