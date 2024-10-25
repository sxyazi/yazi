use std::str::FromStr;

use anyhow::Context;
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
	type Err = anyhow::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		#[derive(Deserialize)]
		struct Outer {
			pick: Pick,
		}

		let outer =
			toml::from_str::<Outer>(s).context("Failed to parse the [pick] section in your yazi.toml")?;

		Ok(outer.pick)
	}
}
