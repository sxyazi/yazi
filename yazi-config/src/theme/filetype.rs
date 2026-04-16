use std::ops::Deref;

use serde::Deserialize;
use yazi_codegen::{DeserializeOver, DeserializeOver2};
use yazi_fs::File;

use super::Is;
use crate::{Pattern, Selectable, Selector, Style};

#[derive(Deserialize, DeserializeOver, DeserializeOver2)]
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
	#[serde(flatten)]
	selector:  Selector,
	#[serde(flatten)]
	pub style: Style,
}

impl Selectable for FiletypeRule {
	fn url_pat(&self) -> Option<&Pattern> { self.selector.url_pat() }

	fn mime_pat(&self) -> Option<&Pattern> { self.selector.mime_pat() }

	fn match_with(&self, file: Option<&File>, mime: Option<&str>) -> bool {
		match (self.is.enabled(), file) {
			(Some(is), Some(f)) if !is.check(&f.cha) => false,
			(Some(_), None) => false,
			_ => self.selector.match_with(file, mime),
		}
	}
}
