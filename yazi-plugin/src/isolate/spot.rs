use std::borrow::Cow;

use mlua::{ExternalError, ExternalResult, HookTriggers, Table, TableExt};
use tokio::{runtime::Handle, select};
use tokio_util::sync::CancellationToken;
use tracing::error;
use yazi_config::LAYOUT;
use yazi_shared::event::Cmd;

use super::slim_lua;
use crate::{bindings::{Cast, Window}, elements::Rect, file::File, loader::LOADER};

pub fn spot(
	cmd: &Cmd,
	file: yazi_shared::fs::File,
	mime: Cow<'static, str>,
	skip: usize,
) -> CancellationToken {
	let ct = CancellationToken::new();

	let name = cmd.name.to_owned();
	let (ct1, ct2) = (ct.clone(), ct.clone());
	tokio::task::spawn_blocking(move || {
		let future = async {
			LOADER.ensure(&name).await.into_lua_err()?;

			let lua = slim_lua(&name)?;
			lua.set_hook(
				HookTriggers::new().on_calls().on_returns().every_nth_instruction(2000),
				move |_, _| {
					if ct1.is_cancelled() { Err("Spot task cancelled".into_lua_err()) } else { Ok(()) }
				},
			);

			let plugin: Table = if let Some(b) = LOADER.read().get(&name) {
				lua.load(b.as_ref()).set_name(name).call(())?
			} else {
				return Err("unloaded plugin".into_lua_err());
			};
			plugin.raw_set("_file", File::cast(&lua, file)?)?;
			plugin.raw_set("_mime", mime)?;
			plugin.raw_set("_skip", skip)?;
			plugin.raw_set("_area", Rect::from(LAYOUT.load().preview))?;
			plugin.raw_set("_window", Window::default())?;

			if ct2.is_cancelled() { Ok(()) } else { plugin.call_async_method("spot", ()).await }
		};

		let result = Handle::current().block_on(async {
			select! {
				_ = ct2.cancelled() => Ok(()),
				r = future => r,
			}
		});

		if let Err(e) = result {
			if !e.to_string().contains("Spot task cancelled") {
				error!("{e:?}");
			}
		}
	});

	ct
}
