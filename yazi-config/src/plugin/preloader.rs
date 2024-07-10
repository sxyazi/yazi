use serde::Deserialize;
use yazi_shared::event::Cmd;

use crate::{Pattern, Priority};

#[derive(Debug, Deserialize)]
pub struct Preloader {
	#[serde(skip)]
	pub idx: u8,

	pub name: Option<Pattern>,
	pub mime: Option<Pattern>,
	pub run:  Cmd,
	#[serde(default)]
	pub next: bool,
	#[serde(default)]
	pub prio: Priority,
}

#[derive(Debug, Clone)]
pub struct PreloaderProps {
	pub id:   u8,
	pub name: String,
	pub prio: Priority,
}

impl From<&Preloader> for PreloaderProps {
	fn from(preloader: &Preloader) -> Self {
		Self { id: preloader.idx, name: preloader.run.name.to_owned(), prio: preloader.prio }
	}
}
