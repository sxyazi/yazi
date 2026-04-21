use serde::Deserialize;
use yazi_codegen::{DeserializeOver, DeserializeOver2, Overlay};
use yazi_fs::File;

use super::FiletypeRules;
use crate::{Selectable, Style};

#[derive(Deserialize, DeserializeOver, DeserializeOver2, Overlay)]
pub struct Filetype {
	rules: FiletypeRules,
}

impl Filetype {
	pub fn match_style(&self, file: &File, mime: &str) -> Option<Style> {
		self.rules.load().iter().find(|rule| rule.matches(file, mime)).map(|rule| rule.style)
	}
}
