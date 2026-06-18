use serde::Deserialize;
use yazi_binding::style::StyleFlat;
use yazi_fs::file::File;

use crate::{Pattern, Selectable};

#[derive(Deserialize)]
pub struct FiletypeRule {
	#[serde(default)]
	is:        super::Is,
	#[serde(flatten)]
	selector:  crate::Selector,
	#[serde(flatten)]
	pub style: StyleFlat,
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
