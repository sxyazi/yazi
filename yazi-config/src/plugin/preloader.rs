use serde::Deserialize;
use yazi_shared::{event::Cmd, Condition};

use crate::{Pattern, Priority};

#[derive(Debug, Deserialize)]
pub struct Preloader {
	#[serde(skip)]
	pub id:    u8,
	pub cond:  Option<Condition>,
	pub name:  Option<Pattern>,
	pub mime:  Option<Pattern>,
	pub run:   Cmd,
	#[serde(default)]
	pub next:  bool,
	#[serde(default)]
	pub multi: bool,
	#[serde(default)]
	pub prio:  Priority,
}

#[derive(Debug, Clone)]
pub struct PreloaderProps {
	pub id:    u8,
	pub name:  String,
	pub multi: bool,
	pub prio:  Priority,
}

impl From<&Preloader> for PreloaderProps {
	fn from(preloader: &Preloader) -> Self {
		Self {
			id:    preloader.id,
			name:  preloader.run.name.to_owned(),
			multi: preloader.multi,
			prio:  preloader.prio,
		}
	}
}
