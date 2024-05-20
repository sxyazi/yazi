use serde::Deserialize;
use yazi_shared::{event::Cmd, Condition};

use crate::{Pattern, Priority};

#[derive(Debug, Deserialize)]
pub struct Prefetcher {
	#[serde(skip)]
	pub id:   u8,
	pub cond: Option<Condition>,
	pub name: Option<Pattern>,
	pub mime: Option<Pattern>,
	pub run:  Cmd,
	#[serde(default)]
	pub prio: Priority,
}

#[derive(Debug, Clone)]
pub struct PrefetcherProps {
	pub id:   u8,
	pub name: String,
	pub prio: Priority,
}

impl From<&Prefetcher> for PrefetcherProps {
	fn from(prefetcher: &Prefetcher) -> Self {
		Self { id: prefetcher.id, name: prefetcher.run.name.to_owned(), prio: prefetcher.prio }
	}
}
