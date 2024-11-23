use std::borrow::Cow;

use mlua::{ExternalError, ExternalResult, HookTriggers, ObjectLike, Table, VmState};
use tokio::{runtime::Handle, select};
use tokio_util::sync::CancellationToken;
use tracing::error;
use yazi_config::LAYOUT;
use yazi_proxy::{AppProxy, options::{PluginCallback, PluginOpt}};
use yazi_shared::event::Cmd;

use super::slim_lua;
use crate::{LUA, bindings::{Cast, Window}, elements::Rect, file::File, loader::LOADER};

pub fn peek(
	cmd: &Cmd,
	file: yazi_shared::fs::File,
	mime: Cow<'static, str>,
	skip: usize,
) -> CancellationToken {
	let ct = CancellationToken::new();
	let (ct1, ct2) = (ct.clone(), ct.clone());

	let name = cmd.name.to_owned();
	tokio::task::spawn_blocking(move || {
		let future = async {
			LOADER.ensure(&name).await.into_lua_err()?;

			let lua = slim_lua(&name)?;
			lua.set_hook(
				HookTriggers::new().on_calls().on_returns().every_nth_instruction(2000),
				move |_, dbg| {
					if ct1.is_cancelled() && dbg.source().what != "C" {
						Err("Peek task cancelled".into_lua_err())
					} else {
						Ok(VmState::Continue)
					}
				},
			);

			let plugin: Table = if let Some(b) = LOADER.read().get(&name) {
				lua.load(b.as_bytes()).set_name(name).call(())?
			} else {
				return Err("unloaded plugin".into_lua_err());
			};
			plugin.raw_set("file", File::cast(&lua, file)?)?;
			plugin.raw_set("mime", mime)?;
			plugin.raw_set("skip", skip)?;
			plugin.raw_set("area", Rect::from(LAYOUT.get().preview))?;
			// TODO: remove this as it's not safe in async context,
			// there may be race conditions between the `window` and `area`
			plugin.raw_set("window", Window::get())?;

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

pub fn peek_sync(cmd: &Cmd, file: yazi_shared::fs::File, mime: Cow<'static, str>, skip: usize) {
	let cb: PluginCallback = Box::new(move |_, plugin| {
		plugin.raw_set("file", File::cast(&LUA, file)?)?;
		plugin.raw_set("mime", mime)?;
		plugin.raw_set("skip", skip)?;
		plugin.raw_set("area", Rect::from(LAYOUT.get().preview))?;
		// TODO: remove this as it's not safe in async context,
		// there may be race conditions between the `window` and `area`
		plugin.raw_set("window", Window::get())?;
		plugin.call_method("peek", ())
	});

	AppProxy::plugin(PluginOpt::new_callback(&cmd.name, cb));
}
