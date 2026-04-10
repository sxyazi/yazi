use std::borrow::Cow;

use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use serde::Deserialize;
use strum::EnumString;
use yazi_shared::{SStr, event::ActionCow, strand::AsStrand};

#[derive(Debug, Deserialize)]
pub struct CopyForm {
	#[serde(alias = "0")]
	pub r#type:    SStr,
	#[serde(default)]
	pub separator: CopySeparator,
	#[serde(default)]
	pub hovered:   bool,
}

impl TryFrom<ActionCow> for CopyForm {
	type Error = anyhow::Error;

	fn try_from(a: ActionCow) -> Result<Self, Self::Error> { Ok(a.deserialize()?) }
}

impl FromLua for CopyForm {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for CopyForm {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}

// --- Separator
#[derive(Clone, Copy, Debug, Default, Deserialize, EnumString, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum CopySeparator {
	#[default]
	Auto,
	Unix,
}

impl CopySeparator {
	pub fn transform<T>(self, s: &T) -> Cow<'_, [u8]>
	where
		T: ?Sized + AsStrand,
	{
		#[cfg(windows)]
		if self == Self::Unix {
			use yazi_shared::strand::StrandCow;
			return match s.as_strand().backslash_to_slash() {
				StrandCow::Borrowed(s) => s.encoded_bytes().into(),
				StrandCow::Owned(s) => s.into_encoded_bytes().into(),
			};
		}
		Cow::Borrowed(s.as_strand().encoded_bytes())
	}
}
