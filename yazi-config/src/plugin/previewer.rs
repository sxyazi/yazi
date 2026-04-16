use std::ops::Deref;

use serde::Deserialize;
use yazi_shared::{Id, event::Action};

use crate::{Mixable, Pattern, Selectable, Selector, plugin::previewer_id};

#[derive(Debug, Deserialize)]
pub struct Previewer {
	#[serde(skip, default = "previewer_id")]
	pub id:       Id,
	#[serde(flatten)]
	pub selector: Selector,
	pub run:      Action,
}

impl Deref for Previewer {
	type Target = Action;

	#[inline]
	fn deref(&self) -> &Self::Target { &self.run }
}

impl Selectable for Previewer {
	fn url_pat(&self) -> Option<&Pattern> { self.selector.url_pat() }

	fn mime_pat(&self) -> Option<&Pattern> { self.selector.mime_pat() }
}

impl Mixable for Previewer {
	fn any_file(&self) -> bool { self.selector.any_file() }

	fn any_dir(&self) -> bool { self.selector.any_dir() }
}
