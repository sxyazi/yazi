use serde::{Deserialize, Serialize};
use strum::IntoStaticStr;

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, IntoStaticStr, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum CdSource {
	#[default]
	Cd,
	Reveal,

	Enter,
	Leave,

	Follow,
	Search,
	Escape,

	Forward,
	Back,

	Tab,
	Displace,
}
