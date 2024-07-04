use mlua::{ExternalError, ExternalResult, HookTriggers, Table, TableExt};
use tokio::{runtime::Handle, select};
use tokio_util::sync::CancellationToken;
use tracing::error;
use yazi_config::LAYOUT;
use yazi_shared::{emit, event::Cmd, Layer};

use super::slim_lua;
use crate::{bindings::{Cast, Window}, elements::Rect, file::File, loader::LOADER, Opt, OptCallback, LUA};

pub fn peek(cmd: &Cmd, file: yazi_shared::fs::File, skip: usize) -> CancellationToken {
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
					if ct1.is_cancelled() { Err("Peek task cancelled".into_lua_err()) } else { Ok(()) }
				},
			);

			let plugin: Table = if let Some(b) = LOADER.read().get(&name) {
				lua.load(b.as_ref()).set_name(name).call(())?
			} else {
				return Err("unloaded plugin".into_lua_err());
			};
			plugin.raw_set("file", File::cast(&lua, file)?)?;
			plugin.raw_set("skip", skip)?;
			plugin.raw_set("area", Rect::cast(&lua, LAYOUT.load().preview)?)?;
			plugin.raw_set("window", Window::default())?;

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
	let cb: OptCallback = Box::new(move |_, plugin| {
		plugin.raw_set("file", File::cast(&LUA, file)?)?;
		plugin.raw_set("skip", skip)?;
		plugin.raw_set("area", Rect::cast(&LUA, LAYOUT.load().preview)?)?;
		plugin.raw_set("window", Window::default())?;
		plugin.call_method("peek", ())
	});

	let cmd: Cmd =
		Opt { name: cmd.name.to_owned(), sync: true, cb: Some(cb), ..Default::default() }.into();

	emit!(Call(cmd.with_name("plugin"), Layer::App));
}
