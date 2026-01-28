use anyhow::Result;
use mlua::ObjectLike;
use scopeguard::defer;
use tracing::{error, warn};
use yazi_binding::runtime_mut;
use yazi_dds::Sendable;
use yazi_macro::succ;
use yazi_parser::app::{PluginMode, PluginOpt};
use yazi_plugin::{LUA, loader::{LOADER, Loader}};
use yazi_proxy::NotifyProxy;
use yazi_shared::data::Data;

use crate::{Actor, Ctx, lives::Lives};

pub struct PluginDo;

impl Actor for PluginDo {
	type Options = PluginOpt;

	const NAME: &str = "plugin_do";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		let loader = LOADER.read();
		let Some(chunk) = loader.get(&*opt.id) else {
			succ!(warn!("plugin `{}` not found", opt.id));
		};

		if let Err(e) = Loader::compatible_or_error(&opt.id, chunk) {
			succ!(NotifyProxy::push_error("Incompatible plugin", e));
		}

		if opt.mode.auto_then(chunk.sync_entry) != PluginMode::Sync {
			succ!(cx.core.tasks.scheduler.plugin_entry(opt));
		}

		let blocking = runtime_mut!(LUA)?.critical_push(&opt.id, true);
		defer! { _ = runtime_mut!(LUA).map(|mut r| r.critical_pop(blocking)) }

		let plugin = match LOADER.load_with(&LUA, &opt.id, chunk) {
			Ok(t) => t,
			Err(e) => succ!(warn!("{e}")),
		};
		drop(loader);

		let result = Lives::scope(&cx.core, || {
			if let Some(cb) = opt.callback {
				cb(&LUA, plugin)
			} else {
				let job = LUA.create_table_from([("args", Sendable::args_to_table(&LUA, opt.args)?)])?;
				plugin.call_method("entry", job)
			}
		});
		if let Err(ref e) = result {
			error!("Sync plugin `{}` failed: {e}", opt.id);
		}
		succ!(result?);
	}
}
