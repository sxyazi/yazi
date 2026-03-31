use mlua::{ExternalResult, ObjectLike};
use tokio::runtime::Handle;
use yazi_dds::Sendable;

use crate::{Runner, loader::LOADER, plugin::PluginOpt};

impl Runner {
	pub async fn entry(&'static self, opt: PluginOpt) -> mlua::Result<()> {
		LOADER.ensure(&opt.id, |_| ()).await.into_lua_err()?;

		tokio::task::spawn_blocking(move || {
			let lua = self.spawn(&opt.id)?;
			let job = lua.create_table_from([("args", Sendable::args_to_table(&lua, opt.args)?)])?;

			Handle::current()
				.block_on(async { LOADER.load(&lua, &opt.id).await?.call_async_method("entry", job).await })
		})
		.await
		.into_lua_err()?
	}
}
