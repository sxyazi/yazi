use anyhow::Result;
use mlua::ObjectLike;
use scopeguard::defer;
use tracing::{error, warn};
use yazi_binding::runtime_mut;
use yazi_core::app::PluginMode;
use yazi_macro::succ;
use yazi_parser::app::PluginForm;
use yazi_plugin::LUA;
use yazi_runner::{entry::EntryJob, loader::{LOADER, Loader}};
use yazi_scheduler::NotifyProxy;
use yazi_shared::data::Data;

use crate::{Actor, Ctx, lives::Lives};

pub struct PluginDo;

impl Actor for PluginDo {
	type Form = PluginForm;

	const NAME: &str = "plugin_do";

	fn act(cx: &mut Ctx, Self::Form { opt }: Self::Form) -> Result<Data> {
		let loader = LOADER.read();
		let Some(chunk) = loader.get(&*opt.id) else {
			succ!(warn!("plugin `{}` not found", opt.id));
		};

		if let Err(e) = Loader::compatible_or_error(&opt.id, chunk) {
			succ!(NotifyProxy::push_error("Incompatible plugin", e.to_string()));
		}

		if opt.mode.auto_then(chunk.sync_entry) != PluginMode::Sync {
			succ!(cx.core.tasks.scheduler.plugin_entry(opt.into()));
		}

		let blocking = runtime_mut!(LUA)?.critical_push(&opt.id, true);
		defer! { _ = runtime_mut!(LUA).map(|mut r| r.critical_pop(blocking)) }

		let plugin = match LOADER.load_chunk(&LUA, &opt.id, chunk) {
			Ok(t) => t,
			Err(e) => succ!(warn!("{e}")),
		};
		drop(loader);

		let result = Lives::scope(cx.core, || {
			if let Some(cb) = opt.callback {
				cb(&LUA, plugin)
			} else {
				plugin.call_method("entry", EntryJob { args: opt.args, ..Default::default() })
			}
		});
		if let Err(ref e) = result {
			error!("Sync plugin `{}` failed: {e}", opt.id);
		}
		succ!(result?);
	}
}
