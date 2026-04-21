use serde::Deserialize;

use crate::{Icon, Mixable, Pattern, Selectable};

#[derive(Debug, Deserialize)]
pub struct IconGlob {
	pub url:  Pattern,
	#[serde(flatten)]
	pub icon: Icon,
}

impl Selectable for IconGlob {
	fn url_pat(&self) -> Option<&Pattern> { Some(&self.url) }

	fn mime_pat(&self) -> Option<&Pattern> { None }
}

impl Mixable for IconGlob {
	fn any_file(&self) -> bool { self.url.any_file() }

	fn any_dir(&self) -> bool { self.url.any_dir() }
}
