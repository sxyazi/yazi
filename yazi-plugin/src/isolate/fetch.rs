use mlua::{ExternalResult, FromLua, IntoLua, Lua, ObjectLike, Value};
use tokio::runtime::Handle;
use yazi_binding::Error;
use yazi_dds::Sendable;
use yazi_shared::event::CmdCow;

use super::slim_lua;
use crate::{file::File, loader::LOADER};

pub async fn fetch(
	cmd: CmdCow,
	files: Vec<yazi_fs::File>,
) -> mlua::Result<(FetchState, Option<Error>)> {
	if files.is_empty() {
		return Ok((FetchState::Bool(true), None));
	}
	LOADER.ensure(&cmd.name, |_| ()).await.into_lua_err()?;

	tokio::task::spawn_blocking(move || {
		let lua = slim_lua(&cmd.name)?;
		let plugin = LOADER.load_once(&lua, &cmd.name)?;

		Handle::current().block_on(plugin.call_async_method(
			"fetch",
			lua.create_table_from([
				("args", Sendable::args_to_table_ref(&lua, &cmd.args)?.into_lua(&lua)?),
				("files", lua.create_sequence_from(files.into_iter().map(File::new))?.into_lua(&lua)?),
			])?,
		))
	})
	.await
	.into_lua_err()?
}

// --- State
pub enum FetchState {
	Bool(bool),
	Vec(Vec<bool>),
}

impl FetchState {
	#[inline]
	pub fn get(&self, idx: usize) -> bool {
		match self {
			Self::Bool(b) => *b,
			Self::Vec(v) => v.get(idx).copied().unwrap_or(false),
		}
	}
}

impl FromLua for FetchState {
	fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
		Ok(match value {
			Value::Boolean(b) => Self::Bool(b),
			Value::Table(tbl) => Self::Vec(tbl.sequence_values().collect::<mlua::Result<_>>()?),
			_ => Err(mlua::Error::FromLuaConversionError {
				from:    value.type_name(),
				to:      "FetchState".to_owned(),
				message: Some("expected a boolean or a table of booleans".to_owned()),
			})?,
		})
	}
}
