use serde::Deserialize;
use serde_with::{OneOrMany, formats::PreferOne, serde_as};

use crate::{Mixable, Pattern, Selectable, Selector};

#[serde_as]
#[derive(Deserialize)]
pub struct OpenRule {
	#[serde(flatten)]
	pub selector: Selector,
	#[serde_as(as = "OneOrMany<_, PreferOne>")]
	pub r#use:    Vec<String>,
}

impl Selectable for OpenRule {
	fn url_pat(&self) -> Option<&Pattern> { self.selector.url_pat() }

	fn mime_pat(&self) -> Option<&Pattern> { self.selector.mime_pat() }
}

impl Mixable for OpenRule {
	fn any_file(&self) -> bool { self.selector.any_file() }

	fn any_dir(&self) -> bool { self.selector.any_dir() }
}
