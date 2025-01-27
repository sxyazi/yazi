use mlua::{ExternalError, ExternalResult, IntoLua, ObjectLike, Table, Value};
use tokio::runtime::Handle;
use yazi_config::LAYOUT;
use yazi_dds::Sendable;
use yazi_shared::event::Cmd;

use super::slim_lua;
use crate::{Error, deprecate, elements::Rect, file::File, loader::LOADER};

pub async fn preload(
	cmd: &'static Cmd,
	file: yazi_fs::File,
) -> mlua::Result<(bool, Option<Error>)> {
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
			("file", File(file).into_lua(&lua)?),
			("skip", 0.into_lua(&lua)?),
		])?;

		let (ok, mut err): (Value, Option<Error>) =
			Handle::current().block_on(plugin.call_async_method("preload", job))?;

		// TODO: remove this
		let ok = match ok {
			Value::Boolean(b) => b,
			Value::Integer(n) => {
				deprecate!(lua, "The integer return value of `preload()` has been deprecated since 25.01.27, please use the new `(bool, error)` instead, in your {}.

See #2253 for more information: https://github.com/sxyazi/yazi/pull/2253");
				if n as u8 & 1 == 0 {
					err = Some(Error::Custom(format!("Returned {n} when running the preloader")));
				}
				n as u8 & 1 == 1
			},
			_ => Err("The first return value of `preload()` must be a bool".into_lua_err())?,
		};

		Ok((ok, err))
	})
	.await
	.into_lua_err()?
}
