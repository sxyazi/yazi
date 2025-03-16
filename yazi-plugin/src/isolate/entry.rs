use mlua::{ExternalError, ExternalResult, ObjectLike, Table};
use tokio::runtime::Handle;
use yazi_dds::Sendable;
use yazi_proxy::options::PluginOpt;

use super::slim_lua;
use crate::loader::LOADER;

pub async fn entry(opt: PluginOpt) -> mlua::Result<()> {
	LOADER.ensure(&opt.id, |_| ()).await.into_lua_err()?;

	tokio::task::spawn_blocking(move || {
		let lua = slim_lua(&opt.id)?;
		let plugin: Table = if let Some(c) = LOADER.read().get(opt.id.as_ref()) {
			lua.load(c.as_bytes()).set_name(opt.id.as_ref()).call(())?
		} else {
			return Err("unloaded plugin".into_lua_err());
		};

		let job = lua.create_table_from([("args", Sendable::args_to_table(&lua, opt.args)?)])?;

		Handle::current().block_on(plugin.call_async_method("entry", job))
	})
	.await
	.into_lua_err()?
}
