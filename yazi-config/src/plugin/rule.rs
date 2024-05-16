use serde::Deserialize;
use yazi_shared::{event::Cmd, Condition};

use crate::{Pattern, Priority};

#[derive(Debug, Deserialize)]
pub struct PluginRule {
	#[serde(skip)]
	pub id:    u8,
	pub cond:  Option<Condition>,
	pub name:  Option<Pattern>,
	pub mime:  Option<Pattern>,
	#[serde(deserialize_with = "super::run_deserialize")]
	pub run:   Cmd,
	#[serde(default)]
	pub sync:  bool,
	#[serde(default)]
	pub multi: bool,
	#[serde(default)]
	pub prio:  Priority,
}

impl PluginRule {
	#[inline]
	pub fn any_file(&self) -> bool { self.name.as_ref().is_some_and(|p| p.any_file()) }

	#[inline]
	pub fn any_dir(&self) -> bool { self.name.as_ref().is_some_and(|p| p.any_dir()) }
}
