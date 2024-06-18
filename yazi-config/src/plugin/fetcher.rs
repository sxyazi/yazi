use serde::Deserialize;
use yazi_shared::{event::Cmd, Condition};

use crate::{Pattern, Priority};

#[derive(Debug, Deserialize)]
pub struct Fetcher {
	#[serde(skip)]
	pub idx: u8,

	pub id:   String,
	pub cond: Option<Condition>,
	pub name: Option<Pattern>,
	pub mime: Option<Pattern>,
	pub run:  Cmd,
	#[serde(default)]
	pub prio: Priority,
}

#[derive(Debug, Clone)]
pub struct FetcherProps {
	pub id:   u8,
	pub name: String,
	pub prio: Priority,
}

impl From<&Fetcher> for FetcherProps {
	fn from(fetcher: &Fetcher) -> Self {
		Self { id: fetcher.idx, name: fetcher.run.name.to_owned(), prio: fetcher.prio }
	}
}
