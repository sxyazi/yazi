use serde::Deserialize;
use strum::{Display, EnumString, FromRepr, IntoStaticStr};

#[derive(
	Clone,
	Copy,
	Debug,
	Default,
	Deserialize,
	Display,
	EnumString,
	Eq,
	FromRepr,
	Hash,
	IntoStaticStr,
	PartialEq,
)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
#[repr(u8)]
pub enum Layer {
	#[default]
	Null,
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
