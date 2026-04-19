use std::hash::{Hash, Hasher};

use serde::Deserialize;
use serde_with::{OneOrMany, formats::PreferOne, serde_as};
use yazi_codegen::DeserializeOver2;
use yazi_shared::Id;

use crate::{Mixable, Pattern, Selectable, Selector, plugin::open_rule_id};

#[serde_as]
#[derive(Clone, Debug, Deserialize, DeserializeOver2)]
pub struct OpenRule {
	#[serde(skip, default = "open_rule_id")]
	pub id:       Id,
	#[serde(flatten)]
	pub selector: Selector,
	#[serde_as(as = "OneOrMany<_, PreferOne>")]
	pub r#use:    Vec<String>,
}

impl PartialEq for OpenRule {
	fn eq(&self, other: &Self) -> bool { self.id == other.id }
}

impl Eq for OpenRule {}

impl Hash for OpenRule {
	fn hash<H: Hasher>(&self, state: &mut H) { self.id.hash(state); }
}

impl Selectable for OpenRule {
	fn url_pat(&self) -> Option<&Pattern> { self.selector.url_pat() }

	fn mime_pat(&self) -> Option<&Pattern> { self.selector.mime_pat() }
}

impl Mixable for OpenRule {
	fn any_file(&self) -> bool { self.selector.any_file() }

	fn any_dir(&self) -> bool { self.selector.any_dir() }
}
