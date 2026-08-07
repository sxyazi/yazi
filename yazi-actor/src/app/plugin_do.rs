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
use yazi_shared::data::{Data, Sendable};

use crate::{Actor, Ctx, lives::Lives};

pub struct PluginDo;

impl Actor for PluginDo {
	type Form = PluginForm;

	const NAME: &str = "plugin_do";

	fn act(cx: &mut Ctx, Self::Form { opt }: Self::Form) -> Result<Data> {
		if opt.scope.is_cancelled() {
			succ!();
		}

		let loader = LOADER.read();
		let Some(chunk) = loader.get(&*opt.name) else {
			succ!(warn!("plugin `{}` not found", opt.name));
		};

		if let Err(e) = Loader::compatible_or_error(&opt.name, chunk) {
			succ!(NotifyProxy::push_error("Incompatible plugin", e.to_string()));
		}

		if opt.mode.auto_then(chunk.sync_entry) != PluginMode::Sync {
			succ!(cx.core.tasks.scheduler.plugin_entry(opt.into()));
		}

		runtime_mut!(LUA)?.enter(&opt.name, true, opt.scope);
		defer! { _ = runtime_mut!(LUA).map(|mut r| r.leave()) }

		let plugin = match LOADER.load_chunk(&LUA, &opt.name, chunk) {
			Ok(t) => t,
			Err(e) => succ!(warn!("{e}")),
		};
		drop(loader);

		let result = Lives::scope(cx.core, |_| {
			if let Some(cb) = opt.callback {
				cb(&LUA, plugin)
			} else if !opt.method.is_empty() {
				plugin.call_method(&opt.method, Sendable::args_to_table(&LUA, opt.args)?)
			} else {
				plugin.call_method("entry", EntryJob { args: opt.args, ..Default::default() })
			}
		});
		if let Err(ref e) = result {
			error!("Sync plugin `{}` failed: {e}", opt.name);
		}
		succ!(result?);
	}
}
