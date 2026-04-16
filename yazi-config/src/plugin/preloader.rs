use std::ops::Deref;

use serde::Deserialize;
use yazi_shared::{Id, event::Action};

use crate::{Mixable, Pattern, Priority, Selectable, Selector, plugin::preloader_id};

#[derive(Debug, Deserialize)]
pub struct Preloader {
	#[serde(skip, default = "preloader_id")]
	pub id:       Id,
	#[serde(skip)]
	pub idx:      u8,
	#[serde(flatten)]
	pub selector: Selector,
	pub run:      Action,
	#[serde(default)]
	pub next:     bool,
	#[serde(default)]
	pub prio:     Priority,
}

impl Deref for Preloader {
	type Target = Action;

	fn deref(&self) -> &Self::Target { &self.run }
}

impl Selectable for Preloader {
	fn url_pat(&self) -> Option<&Pattern> { self.selector.url_pat() }

	fn mime_pat(&self) -> Option<&Pattern> { self.selector.mime_pat() }
}

impl Mixable for Preloader {}
