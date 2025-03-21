use std::ops::Deref;

use serde::Deserialize;
use yazi_codegen::DeserializeOver2;
use yazi_fs::File;
use yazi_shared::theme::Style;

use super::Is;
use crate::Pattern;

#[derive(Deserialize, DeserializeOver2)]
pub struct Filetype {
	rules: Vec<FiletypeRule>,
}

impl Deref for Filetype {
	type Target = Vec<FiletypeRule>;

	fn deref(&self) -> &Self::Target { &self.rules }
}

#[derive(Deserialize)]
pub struct FiletypeRule {
	#[serde(default)]
	is:        Is,
	name:      Option<Pattern>,
	mime:      Option<Pattern>,
	#[serde(flatten)]
	pub style: Style,
}

impl FiletypeRule {
	pub fn matches(&self, file: &File, mime: &str) -> bool {
		if !self.is.check(&file.cha) {
			return false;
		}

		self.mime.as_ref().is_some_and(|p| p.match_mime(mime))
			|| self.name.as_ref().is_some_and(|n| n.match_path(&file.url, file.is_dir()))
	}
}
