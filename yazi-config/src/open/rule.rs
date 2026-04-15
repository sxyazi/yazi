use serde::Deserialize;
use serde_with::{OneOrMany, formats::PreferOne, serde_as};

use crate::{Mixable, pattern::Pattern};

#[serde_as]
#[derive(Debug, Deserialize)]
pub struct OpenRule {
	pub url:   Option<Pattern>,
	pub mime:  Option<Pattern>,
	#[serde_as(as = "OneOrMany<_, PreferOne>")]
	pub r#use: Vec<String>,
}

impl Mixable for OpenRule {
	fn any_file(&self) -> bool { self.url.as_ref().is_some_and(|p| p.any_file()) }

	fn any_dir(&self) -> bool { self.url.as_ref().is_some_and(|p| p.any_dir()) }
}
