use std::ops::Deref;

use serde::Deserialize;
use yazi_shared::{Id, event::Action};

use crate::{Mixable, Pattern, Selectable, Selector, plugin::spotter_id};

#[derive(Debug, Deserialize)]
pub struct Spotter {
	#[serde(skip, default = "spotter_id")]
	pub id:       Id,
	#[serde(flatten)]
	pub selector: Selector,
	pub run:      Action,
}

impl Deref for Spotter {
	type Target = Action;

	#[inline]
	fn deref(&self) -> &Self::Target { &self.run }
}

impl Selectable for Spotter {
	fn url_pat(&self) -> Option<&Pattern> { self.selector.url_pat() }

	fn mime_pat(&self) -> Option<&Pattern> { self.selector.mime_pat() }
}

impl Mixable for Spotter {
	fn any_file(&self) -> bool { self.selector.any_file() }

	fn any_dir(&self) -> bool { self.selector.any_dir() }
}
