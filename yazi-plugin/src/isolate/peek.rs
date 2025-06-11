use std::borrow::Cow;

use mlua::{ExternalError, HookTriggers, IntoLua, ObjectLike, VmState};
use tokio::{runtime::Handle, select};
use tokio_util::sync::CancellationToken;
use tracing::error;
use yazi_config::LAYOUT;
use yazi_dds::Sendable;
use yazi_proxy::{AppProxy, options::{PluginCallback, PluginOpt}};
use yazi_shared::event::Cmd;

use super::slim_lua;
use crate::{elements::Rect, file::File, loader::LOADER};

pub fn peek(
	cmd: &'static Cmd,
	file: yazi_fs::File,
	mime: Cow<'static, str>,
	skip: usize,
) -> Option<CancellationToken> {
	let ct = CancellationToken::new();
	if let Some(c) = LOADER.read().get(cmd.name.as_ref()) {
		if c.sync_peek {
			peek_sync(cmd, file, mime, skip);
		} else {
			peek_async(cmd, file, mime, skip, ct.clone());
		}
		return Some(ct).filter(|_| !c.sync_peek);
	}

	let ct_ = ct.clone();
	tokio::spawn(async move {
		select! {
			_ = ct_.cancelled() => {},
			Ok(b) = LOADER.ensure(&cmd.name, |c| c.sync_peek) => {
				if b {
					peek_sync( cmd, file, mime, skip);
				} else {
					peek_async( cmd, file, mime, skip, ct_);
				}
			},
			else => {}
		}
	});

	Some(ct)
}

fn peek_sync(cmd: &'static Cmd, file: yazi_fs::File, mime: Cow<'static, str>, skip: usize) {
	let cb: PluginCallback = Box::new(move |lua, plugin| {
		let job = lua.create_table_from([
			("area", Rect::from(LAYOUT.get().preview).into_lua(lua)?),
			("args", Sendable::args_to_table_ref(lua, &cmd.args)?.into_lua(lua)?),
			("file", File::new(file).into_lua(lua)?),
			("mime", mime.into_lua(lua)?),
			("skip", skip.into_lua(lua)?),
		])?;

		plugin.call_method("peek", job)
	});

	AppProxy::plugin(PluginOpt::new_callback(cmd.name.as_ref(), cb));
}

fn peek_async(
	cmd: &'static Cmd,
	file: yazi_fs::File,
	mime: Cow<'static, str>,
	skip: usize,
	ct: CancellationToken,
) {
	let ct_ = ct.clone();
	tokio::task::spawn_blocking(move || {
		let future = async {
			let lua = slim_lua(&cmd.name)?;
			lua.set_hook(
				HookTriggers::new().on_calls().on_returns().every_nth_instruction(2000),
				move |_, dbg| {
					if ct.is_cancelled() && dbg.source().what != "C" {
						Err("Peek task cancelled".into_lua_err())
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
				("mime", mime.into_lua(&lua)?),
				("skip", skip.into_lua(&lua)?),
			])?;

			if ct_.is_cancelled() { Ok(()) } else { plugin.call_async_method("peek", job).await }
		};

		let result = Handle::current().block_on(async {
			select! {
				_ = ct_.cancelled() => Ok(()),
				r = future => r,
			}
		});

		if let Err(e) = result {
			if !e.to_string().contains("Peek task cancelled") {
				error!("{e}");
			}
		}
	});
}
