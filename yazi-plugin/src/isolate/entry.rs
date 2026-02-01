use mlua::{ExternalResult, ObjectLike};
use tokio::runtime::Handle;
use yazi_dds::Sendable;
use yazi_parser::app::PluginOpt;

use super::slim_lua;
use crate::loader::LOADER;

pub async fn entry(opt: PluginOpt) -> mlua::Result<()> {
	LOADER.ensure(&opt.id, |_| ()).await.into_lua_err()?;

	tokio::task::spawn_blocking(move || {
		let lua = slim_lua(&opt.id)?;
		let job = lua.create_table_from([("args", Sendable::args_to_table(&lua, opt.args)?)])?;

		Handle::current()
			.block_on(async { LOADER.load(&lua, &opt.id).await?.call_async_method("entry", job).await })
	})
	.await
	.into_lua_err()?
}
