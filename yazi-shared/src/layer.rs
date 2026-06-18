use mlua::{MetaMethod, UserData, UserDataMethods};
use serde::Deserialize;
use strum::{Display, EnumString, FromRepr, IntoStaticStr};
use yazi_shim::strum::IntoStr;

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

impl UserData for Layer {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_meta_method(MetaMethod::ToString, |_, me, ()| Ok(me.into_str()));
	}
}
