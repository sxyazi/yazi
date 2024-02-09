use mlua::{ExternalError, ExternalResult, HookTriggers, Table, TableExt};
use tokio::{runtime::Handle, select};
use tokio_util::sync::CancellationToken;
use tracing::error;
use yazi_config::LAYOUT;
use yazi_shared::{emit, event::Cmd, Layer};

use super::slim_lua;
use crate::{bindings::{Cast, File, Window}, elements::Rect, OptData, LOADED, LUA};

pub fn peek(cmd: &Cmd, file: yazi_shared::fs::File, skip: usize) -> CancellationToken {
	let ct = CancellationToken::new();

	let name = cmd.name.to_owned();
	let (ct1, ct2) = (ct.clone(), ct.clone());
	tokio::task::spawn_blocking(move || {
		let future = async {
			LOADED.ensure(&name).await.into_lua_err()?;

			let lua = slim_lua()?;
			lua.set_hook(
				HookTriggers::new().on_calls().on_returns().every_nth_instruction(2000),
				move |_, _| {
					if ct1.is_cancelled() { Err("Peek task cancelled".into_lua_err()) } else { Ok(()) }
				},
			);

			let plugin: Table = if let Some(b) = LOADED.read().get(&name) {
				lua.load(b).call(())?
			} else {
				return Err("unloaded plugin".into_lua_err());
			};
			plugin.set("file", File::cast(&lua, file)?)?;
			plugin.set("skip", skip)?;
			plugin.set("area", Rect::cast(&lua, LAYOUT.load().preview)?)?;
			plugin.set("window", Window::default())?;

			if ct2.is_cancelled() { Ok(()) } else { plugin.call_async_method("peek", ()).await }
		};

		let result = Handle::current().block_on(async {
			select! {
				_ = ct2.cancelled() => Ok(()),
				r = future => r,
			}
		});

		if let Err(e) = result {
			if !e.to_string().contains("Peek task cancelled") {
				error!("{e}");
			}
		}
	});

	ct
}

pub fn peek_sync(cmd: &Cmd, file: yazi_shared::fs::File, skip: usize) {
	let data = OptData {
		cb: Some(Box::new(move |_, plugin| {
			plugin.set("file", File::cast(&LUA, file)?)?;
			plugin.set("skip", skip)?;
			plugin.set("area", Rect::cast(&LUA, LAYOUT.load().preview)?)?;
			plugin.set("window", Window::default())?;
			plugin.call_method("peek", ())
		})),
		..Default::default()
	};
	emit!(Call(
		Cmd::args("plugin", vec![cmd.name.to_owned()]).with_bool("sync", true).with_data(data),
		Layer::App
	));
}
