use mlua::{ExternalError, ExternalResult, IntoLua, ObjectLike, Table};
use tokio::runtime::Handle;
use yazi_config::LAYOUT;
use yazi_dds::Sendable;
use yazi_shared::event::CmdCow;

use super::slim_lua;
use crate::{bindings::Cast, elements::Rect, file::File, loader::LOADER};

pub async fn fetch(cmd: CmdCow, files: Vec<yazi_fs::File>) -> mlua::Result<u8> {
	if files.is_empty() {
		return Ok(1);
	}
	LOADER.ensure(&cmd.name).await.into_lua_err()?;

	tokio::task::spawn_blocking(move || {
		let lua = slim_lua(&cmd.name)?;
		let plugin: Table = if let Some(b) = LOADER.read().get(&cmd.name) {
			lua.load(b.as_bytes()).set_name(&cmd.name).call(())?
		} else {
			return Err("unloaded plugin".into_lua_err());
		};

		let files =
			lua.create_sequence_from(files.into_iter().filter_map(|f| File::cast(&lua, f).ok()))?;

		if files.raw_len() == 0 {
			return Ok(1);
		}

		Handle::current().block_on(plugin.call_async_method(
			"fetch",
			lua.create_table_from([
				("area", Rect::from(LAYOUT.get().preview).into_lua(&lua)?),
				("args", Sendable::args_to_table_ref(&lua, &cmd.args)?.into_lua(&lua)?),
				("files", files.into_lua(&lua)?),
			])?,
		))
	})
	.await
	.into_lua_err()?
}
