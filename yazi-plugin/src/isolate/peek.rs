use std::borrow::Cow;

use mlua::{ExternalError, ExternalResult, HookTriggers, IntoLua, ObjectLike, Table, VmState};
use tokio::{runtime::Handle, select};
use tokio_util::sync::CancellationToken;
use tracing::error;
use yazi_config::LAYOUT;
use yazi_dds::Sendable;
use yazi_proxy::{AppProxy, options::{PluginCallback, PluginOpt}};
use yazi_shared::event::Cmd;

use super::slim_lua;
use crate::{bindings::Cast, elements::Rect, file::File, loader::LOADER};

pub fn peek(
	cmd: &'static Cmd,
	file: yazi_fs::File,
	mime: Cow<'static, str>,
	skip: usize,
) -> CancellationToken {
	let ct = CancellationToken::new();
	let (ct1, ct2) = (ct.clone(), ct.clone());

	tokio::task::spawn_blocking(move || {
		let future = async {
			LOADER.ensure(&cmd.name).await.into_lua_err()?;

			let lua = slim_lua(&cmd.name)?;
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

			let plugin: Table = if let Some(b) = LOADER.read().get(&cmd.name) {
				lua.load(b.as_bytes()).set_name(&cmd.name).call(())?
			} else {
				return Err("unloaded plugin".into_lua_err());
			};

			let job = lua.create_table_from([
				("area", Rect::from(LAYOUT.get().preview).into_lua(&lua)?),
				("args", Sendable::args_to_table_ref(&lua, &cmd.args)?.into_lua(&lua)?),
				("file", File::cast(&lua, file)?.into_lua(&lua)?),
				("mime", mime.into_lua(&lua)?),
				("skip", skip.into_lua(&lua)?),
			])?;

			if ct2.is_cancelled() { Ok(()) } else { plugin.call_async_method("peek", job).await }
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

pub fn peek_sync(cmd: &'static Cmd, file: yazi_fs::File, mime: Cow<'static, str>, skip: usize) {
	let cb: PluginCallback = Box::new(move |lua, plugin| {
		let job = lua.create_table_from([
			("area", Rect::from(LAYOUT.get().preview).into_lua(lua)?),
			("args", Sendable::args_to_table_ref(lua, &cmd.args)?.into_lua(lua)?),
			("file", File::cast(lua, file)?.into_lua(lua)?),
			("mime", mime.into_lua(lua)?),
			("skip", skip.into_lua(lua)?),
		])?;

		plugin.call_method("peek", job)
	});

	AppProxy::plugin(PluginOpt::new_callback(&cmd.name, cb));
}
