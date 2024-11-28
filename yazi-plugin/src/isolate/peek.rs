use std::{borrow::Cow, time::Duration};

use mlua::{ExternalError, ExternalResult, HookTriggers, IntoLua, Lua, MetaMethod, MultiValue, ObjectLike, Table, VmState};
use tokio::{runtime::Handle, select};
use tokio_util::sync::CancellationToken;
use tracing::error;
use yazi_config::LAYOUT;
use yazi_dds::Sendable;
use yazi_proxy::{AppProxy, options::{PluginCallback, PluginOpt}};
use yazi_shared::event::Cmd;

use super::slim_lua;
use crate::{RtRef, bindings::Cast, elements::Rect, file::File, loader::LOADER};

pub fn peek(
	cmd: &'static Cmd,
	file: yazi_shared::fs::File,
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
				("args", Sendable::dict_to_table_ref(&lua, &cmd.args)?.into_lua(&lua)?),
				("file", File::cast(&lua, file)?.into_lua(&lua)?),
				("mime", mime.into_lua(&lua)?),
				("skip", skip.into_lua(&lua)?),
			])?;

			// TODO: remove this
			install_warn_mt(&lua, &plugin, job.clone()).ok();

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

pub fn peek_sync(
	cmd: &'static Cmd,
	file: yazi_shared::fs::File,
	mime: Cow<'static, str>,
	skip: usize,
) {
	let cb: PluginCallback = Box::new(move |lua, plugin| {
		let job = lua.create_table_from([
			("area", Rect::from(LAYOUT.get().preview).into_lua(lua)?),
			("args", Sendable::dict_to_table_ref(lua, &cmd.args)?.into_lua(lua)?),
			("file", File::cast(lua, file)?.into_lua(lua)?),
			("mime", mime.into_lua(lua)?),
			("skip", skip.into_lua(lua)?),
		])?;

		// TODO: remove this
		install_warn_mt(lua, &plugin, job.clone()).ok();

		plugin.call_method("peek", job)
	});

	AppProxy::plugin(PluginOpt::new_callback(&cmd.name, cb));
}

pub(super) fn install_warn_mt(lua: &Lua, plugin: &Table, job: Table) -> mlua::Result<()> {
	let mt = lua.create_table_from([(
		MetaMethod::Index.name(),
		lua.create_function(|lua, (ts, key): (Table, mlua::String)| {
			let b = key.as_bytes() == b"file"
				|| key.as_bytes() == b"mime"
				|| key.as_bytes() == b"skip"
				|| key.as_bytes() == b"area";
			if b {
				warn_deprecated(lua.named_registry_value::<RtRef>("rt")?.current());
			}
			lua
				.load(mlua::chunk! {
					if $b then
						return rawget($ts, "__unsafe_job")[$key]
					else
						return rawget($ts, $key)
					end
				})
				.call::<MultiValue>(())
		})?,
	)])?;
	plugin.set_metatable(Some(mt));
	plugin.raw_set("__unsafe_job", job.clone())?;
	Ok(())
}

#[inline]
fn warn_deprecated(id: Option<&str>) {
	static WARNED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
	if !WARNED.swap(true, std::sync::atomic::Ordering::Relaxed) {
		let id = match id {
			Some(id) => format!("`{id}.yazi` plugin"),
			None => "`init.lua` config".to_owned(),
		};
		let s = "The file list for peek() and preload() method has been moved from `self` to the first parameter `job` of the method to avoid conflicts with the plugin's own attributes.

Please use the new `job` parameter in your {id} instead of `self`. See #1966 for details: https://github.com/sxyazi/yazi/pull/1966";
		yazi_proxy::AppProxy::notify(yazi_proxy::options::NotifyOpt {
			title:   "Deprecated API".to_owned(),
			content: s.replace("{id}", &id),
			level:   yazi_proxy::options::NotifyLevel::Warn,
			timeout: Duration::from_secs(20),
		});
	}
}
