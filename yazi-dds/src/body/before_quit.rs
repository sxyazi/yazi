use mlua::{IntoLua, Lua, Value};
use serde::{Deserialize, Serialize};
use yazi_parser::mgr::QuitOpt;
use yazi_shared::Source;

use crate::{body::Body, local_or_err};

#[derive(Debug, Serialize, Deserialize)]
pub struct BeforeQuitBody {
	pub opts:   QuitOpt,
	pub source: Source,
}

impl IntoLua for BeforeQuitBody {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		lua
			.create_table_from([
				("opts", self.opts.into_lua(lua)?),
				("source", yazi_binding::Source(self.source).into_lua(lua)?),
			])?
			.into_lua(lua)
	}
}

impl TryFrom<&QuitOpt> for Body<'static> {
	type Error = anyhow::Error;

	fn try_from(value: &QuitOpt) -> Result<Self, Self::Error> {
		local_or_err!("before-quit");

		Ok(Body::BeforeQuit(BeforeQuitBody {
			opts:   value.clone(),
			source: Source::default(), // FIXME
		}))
	}
}
