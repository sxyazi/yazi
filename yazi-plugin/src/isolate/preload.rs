use mlua::{ExternalError, ExternalResult, IntoLua, ObjectLike, Table};
use tokio::runtime::Handle;
use yazi_config::LAYOUT;
use yazi_dds::Sendable;
use yazi_shared::event::Cmd;

use super::slim_lua;
use crate::{bindings::Cast, elements::Rect, file::File, loader::LOADER};

pub async fn preload(cmd: &'static Cmd, file: yazi_fs::File) -> mlua::Result<u8> {
	LOADER.ensure(&cmd.name).await.into_lua_err()?;

	tokio::task::spawn_blocking(move || {
		let lua = slim_lua(&cmd.name)?;
		let plugin: Table = if let Some(b) = LOADER.read().get(&cmd.name) {
			lua.load(b.as_bytes()).set_name(&cmd.name).call(())?
		} else {
			return Err("unloaded plugin".into_lua_err());
		};

		let job = lua.create_table_from([
			("area", Rect::from(LAYOUT.get().preview).into_lua(&lua)?),
			("args", Sendable::args_to_table_ref(&lua, &cmd.args)?.into_lua(&lua)?),
			("file", File::cast(&lua, file)?.into_lua(&lua)?),
			("skip", 0.into_lua(&lua)?),
		])?;

		// TODO: remove this
		super::install_warn_mt(&lua, &plugin, job.clone()).ok();

		Handle::current().block_on(plugin.call_async_method("preload", job))
	})
	.await
	.into_lua_err()?
}
