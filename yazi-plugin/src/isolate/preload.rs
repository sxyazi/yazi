use mlua::{ExternalResult, IntoLua, ObjectLike};
use tokio::runtime::Handle;
use yazi_binding::Error;
use yazi_config::LAYOUT;
use yazi_dds::Sendable;
use yazi_shared::event::Cmd;

use super::slim_lua;
use crate::{elements::Rect, file::File, loader::LOADER};

pub async fn preload(
	cmd: &'static Cmd,
	file: yazi_fs::File,
) -> mlua::Result<(bool, Option<Error>)> {
	LOADER.ensure(&cmd.name, |_| ()).await.into_lua_err()?;

	tokio::task::spawn_blocking(move || {
		let lua = slim_lua(&cmd.name)?;
		let plugin = LOADER.load_once(&lua, &cmd.name)?;

		let job = lua.create_table_from([
			("area", Rect::from(LAYOUT.get().preview).into_lua(&lua)?),
			("args", Sendable::args_to_table_ref(&lua, &cmd.args)?.into_lua(&lua)?),
			("file", File::new(file).into_lua(&lua)?),
			("skip", 0.into_lua(&lua)?),
		])?;

		Handle::current().block_on(plugin.call_async_method("preload", job))
	})
	.await
	.into_lua_err()?
}
