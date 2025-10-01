use std::{borrow::Cow, ffi::OsStr, path::Path, str::FromStr};

use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use serde::Deserialize;
use yazi_shared::{SStr, event::CmdCow};

#[derive(Debug)]
pub struct CopyOpt {
	pub r#type:    SStr,
	pub separator: CopySeparator,
	pub hovered:   bool,
}

impl From<CmdCow> for CopyOpt {
	fn from(mut c: CmdCow) -> Self {
		Self {
			r#type:    c.take_first().unwrap_or_default(),
			separator: c.str("separator").parse().unwrap_or_default(),
			hovered:   c.bool("hovered"),
		}
	}
}

impl FromLua for CopyOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for CopyOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}

// --- Separator
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CopySeparator {
	#[default]
	Auto,
	Unix,
}

impl FromStr for CopySeparator {
	type Err = serde::de::value::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Self::deserialize(serde::de::value::StrDeserializer::new(s))
	}
}

impl CopySeparator {
	pub fn transform<T: AsRef<Path> + ?Sized>(self, p: &T) -> Cow<'_, OsStr> {
		#[cfg(windows)]
		if self == Self::Unix {
			return match yazi_fs::path::backslash_to_slash(p.as_ref()) {
				Cow::Owned(p) => Cow::Owned(p.into_os_string()),
				Cow::Borrowed(p) => Cow::Borrowed(p.as_os_str()),
			};
		}
		Cow::Borrowed(p.as_ref().as_os_str())
	}
}
