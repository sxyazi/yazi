use std::ops::Deref;

use serde::Deserialize;
use yazi_shared::{Id, event::Action};

use crate::{Mixable, Pattern, Priority, Selectable, Selector, plugin::fetcher_id};

#[derive(Debug, Deserialize)]
pub struct Fetcher {
	#[serde(skip, default = "fetcher_id")]
	pub id:       Id,
	#[serde(skip)]
	pub idx:      u8,
	#[serde(flatten)]
	pub selector: Selector,
	pub run:      Action,
	#[serde(default)]
	pub prio:     Priority,
	pub group:    String,
}

impl Deref for Fetcher {
	type Target = Action;

	fn deref(&self) -> &Self::Target { &self.run }
}

impl Selectable for Fetcher {
	fn url_pat(&self) -> Option<&Pattern> { self.selector.url_pat() }

	fn mime_pat(&self) -> Option<&Pattern> { self.selector.mime_pat() }
}

impl Mixable for Fetcher {}
