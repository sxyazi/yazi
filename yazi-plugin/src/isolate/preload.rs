use mlua::{ExternalError, ExternalResult, HookTriggers, IntoLua, ObjectLike, VmState};
use tokio::{runtime::Handle, select};
use tokio_util::sync::CancellationToken;
use yazi_binding::Error;
use yazi_config::LAYOUT;
use yazi_dds::Sendable;
use yazi_shared::event::Cmd;

use super::slim_lua;
use crate::{elements::Rect, file::File, loader::LOADER};

pub async fn preload(
	cmd: &'static Cmd,
	file: yazi_fs::File,
	ct: CancellationToken,
) -> mlua::Result<(bool, Option<Error>)> {
	let ct_ = ct.clone();
	tokio::task::spawn_blocking(move || {
		let future = async {
			LOADER.ensure(&cmd.name, |_| ()).await.into_lua_err()?;

			let lua = slim_lua(&cmd.name)?;
			lua.set_hook(
				HookTriggers::new().on_calls().on_returns().every_nth_instruction(2000),
				move |_, dbg| {
					if ct.is_cancelled() && dbg.source().what != "C" {
						Err("Preload task cancelled".into_lua_err())
					} else {
						Ok(VmState::Continue)
					}
				},
			);

			let plugin = LOADER.load_once(&lua, &cmd.name)?;
			let job = lua.create_table_from([
				("area", Rect::from(LAYOUT.get().preview).into_lua(&lua)?),
				("args", Sendable::args_to_table_ref(&lua, &cmd.args)?.into_lua(&lua)?),
				("file", File::new(file).into_lua(&lua)?),
				("skip", 0.into_lua(&lua)?),
			])?;

			if ct_.is_cancelled() {
				Ok((false, None))
			} else {
				plugin.call_async_method("preload", job).await
			}
		};

		Handle::current().block_on(async {
			select! {
				_ = ct_.cancelled() => Ok((false, None)),
				r = future => match r {
					Err(e) if e.to_string().contains("Preload task cancelled") => Ok((false, None)),
					Ok(_) | Err(_) => r,
				},
			}
		})
	})
	.await
	.into_lua_err()?
}
