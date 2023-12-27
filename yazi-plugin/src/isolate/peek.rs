use mlua::{ExternalError, ExternalResult, HookTriggers, Table, TableExt};
use tokio::{runtime::Handle, select};
use tokio_util::sync::CancellationToken;
use tracing::error;
use yazi_config::LAYOUT;
use yazi_shared::{emit, event::Exec, Layer};

use super::slim_lua;
use crate::{bindings::{Cast, File, Window}, elements::Rect, OptData, LOADED, LUA};

pub fn peek(exec: &Exec, file: yazi_shared::fs::File, skip: usize) -> CancellationToken {
	let ct = CancellationToken::new();

	let cmd = exec.cmd.to_owned();
	let ct2 = ct.clone();
	tokio::task::spawn_blocking(move || {
		let future = async {
			LOADED.ensure(&cmd).await.into_lua_err()?;

			let lua = slim_lua()?;
			// FIXME: this will cause a panic
			// lua.set_hook(
			// 	HookTriggers::new().on_calls().on_returns().every_nth_instruction(2000),
			// 	move |_, _| {
			// 		if ct1.is_cancelled() { Err("cancelled".into_lua_err()) } else { Ok(()) }
			// 	},
			// );

			let plugin: Table = if let Some(b) = LOADED.read().get(&cmd) {
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
			error!("{e}");
		}
	});

	ct
}

pub fn peek_sync(exec: &Exec, file: yazi_shared::fs::File, skip: usize) {
	let data = OptData {
		args: vec![],
		cb:   Some(Box::new(move |plugin| {
			plugin.set("file", File::cast(&LUA, file)?)?;
			plugin.set("skip", skip)?;
			plugin.set("area", Rect::cast(&LUA, LAYOUT.load().preview)?)?;
			plugin.set("window", Window::default())?;
			plugin.call_method("peek", ())
		})),
		tx:   None,
	};
	emit!(Call(
		Exec::call("plugin", vec![exec.cmd.to_owned()]).with_bool("sync", true).with_data(data).vec(),
		Layer::App
	));
}
