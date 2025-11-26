use std::str::FromStr;

use anyhow::bail;
use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use serde::Deserialize;
use yazi_shared::{SStr, event::CmdCow};

#[derive(Debug)]
pub struct SearchOpt {
	pub via:      SearchOptVia,
	pub subject:  SStr,
	pub args:     Vec<String>,
	pub args_raw: SStr,
}

impl TryFrom<CmdCow> for SearchOpt {
	type Error = anyhow::Error;

	fn try_from(mut c: CmdCow) -> Result<Self, Self::Error> {
		// TODO: remove this
		let (via, subject) = if let Ok(s) = c.get("via") {
			(str::parse(s)?, c.take_first().unwrap_or_default())
		} else {
			(c.str(0).parse()?, "".into())
		};

		let Ok(args) = yazi_shared::shell::unix::split(c.str("args"), false) else {
			bail!("Invalid 'args' argument in SearchOpt");
		};

		Ok(Self {
			via,
			subject,
			// TODO: use second positional argument instead of `args` parameter
			args: args.0,
			args_raw: c.take("args").unwrap_or_default(),
		})
	}
}

impl FromLua for SearchOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for SearchOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}

// Via
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum SearchOptVia {
	Rg,
	Rga,
	Fd,
}

impl FromStr for SearchOptVia {
	type Err = serde::de::value::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Self::deserialize(serde::de::value::StrDeserializer::new(s))
	}
}

impl SearchOptVia {
	pub fn into_str(self) -> &'static str {
		match self {
			Self::Rg => "rg",
			Self::Rga => "rga",
			Self::Fd => "fd",
		}
	}
}
