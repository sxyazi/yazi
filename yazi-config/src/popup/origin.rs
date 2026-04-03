use serde::Deserialize;
use strum::{EnumString, IntoStaticStr};

#[derive(Clone, Copy, Debug, Default, Deserialize, EnumString, Eq, IntoStaticStr, PartialEq)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum Origin {
	#[default]
	TopLeft,
	TopCenter,
	TopRight,

	BottomLeft,
	BottomCenter,
	BottomRight,

	Center,
	Hovered,
}
