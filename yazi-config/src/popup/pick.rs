use std::str::FromStr;

use serde::Deserialize;

use super::{Offset, Origin};

#[derive(Deserialize)]
pub struct Pick {
	// open
	pub open_title:  String,
	pub open_origin: Origin,
	pub open_offset: Offset,
}

impl Pick {
	pub const fn border(&self) -> u16 { 2 }
}

impl FromStr for Pick {
	type Err = toml::de::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		#[derive(Deserialize)]
		struct Outer {
			pick: Pick,
		}

		Ok(toml::from_str::<Outer>(s)?.pick)
	}
}
