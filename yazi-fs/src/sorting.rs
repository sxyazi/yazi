use serde::{Deserialize, Serialize};
use strum::{EnumString, IntoStaticStr};

// --- by
#[derive(
	Clone, Copy, Debug, Default, Deserialize, EnumString, Eq, IntoStaticStr, PartialEq, Serialize,
)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum SortBy {
	#[default]
	None,
	Mtime,
	Btime,
	Extension,
	Alphabetical,
	Natural,
	Size,
	Random,
}

// --- fallback
#[derive(
	Clone, Copy, Debug, Default, Deserialize, EnumString, Eq, IntoStaticStr, PartialEq, Serialize,
)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum SortFallback {
	#[default]
	Alphabetical,
	Natural,
}
