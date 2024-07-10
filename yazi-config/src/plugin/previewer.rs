use serde::Deserialize;
use yazi_shared::event::Cmd;

use crate::Pattern;

#[derive(Debug, Deserialize)]
pub struct Previewer {
	pub name: Option<Pattern>,
	pub mime: Option<Pattern>,
	pub run:  Cmd,
	#[serde(default)]
	pub sync: bool,
}

impl Previewer {
	#[inline]
	pub fn any_file(&self) -> bool { self.name.as_ref().is_some_and(|p| p.any_file()) }

	#[inline]
	pub fn any_dir(&self) -> bool { self.name.as_ref().is_some_and(|p| p.any_dir()) }
}
