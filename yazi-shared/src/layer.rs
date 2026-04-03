use serde::Deserialize;
use strum::{Display, EnumString, IntoStaticStr};

#[derive(
	Clone, Copy, Debug, Default, Deserialize, Display, EnumString, Eq, Hash, IntoStaticStr, PartialEq,
)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum Layer {
	#[default]
	App,
	Mgr,
	Tasks,
	Spot,
	Pick,
	Input,
	Confirm,
	Help,
	Cmp,
	Which,
	Notify,
}
