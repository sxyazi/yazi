use anyhow::Result;
use mlua::ObjectLike;
use scopeguard::defer;
use tracing::{error, warn};
use yazi_actor::lives::Lives;
use yazi_binding::runtime_mut;
use yazi_dds::Sendable;
use yazi_macro::succ;
use yazi_parser::app::{PluginMode, PluginOpt};
use yazi_plugin::{LUA, loader::{LOADER, Loader}};
use yazi_proxy::AppProxy;
use yazi_shared::data::Data;

use crate::app::App;

impl App {
	pub(crate) fn plugin(&mut self, mut opt: PluginOpt) -> Result<Data> {
		let mut hits = false;
		if let Some(chunk) = LOADER.read().get(&*opt.id) {
			hits = true;
			opt.mode = opt.mode.auto_then(chunk.sync_entry);
		}

		if opt.mode == PluginMode::Async {
			succ!(self.core.tasks.plugin_entry(opt));
		} else if opt.mode == PluginMode::Sync && hits {
			return self.plugin_do(opt);
		}

		tokio::spawn(async move {
			match LOADER.ensure(&opt.id, |_| ()).await {
				Ok(()) => AppProxy::plugin_do(opt),
				Err(e) => AppProxy::notify_error("Plugin load failed", e),
			}
		});
		succ!();
	}

	pub(crate) fn plugin_do(&mut self, opt: PluginOpt) -> Result<Data> {
		let loader = LOADER.read();
		let Some(chunk) = loader.get(&*opt.id) else {
			succ!(warn!("plugin `{}` not found", opt.id));
		};

		if let Err(e) = Loader::compatible_or_error(&opt.id, chunk) {
			succ!(AppProxy::notify_error("Incompatible plugin", e));
		}

		if opt.mode.auto_then(chunk.sync_entry) != PluginMode::Sync {
			succ!(self.core.tasks.plugin_entry(opt));
		}

		runtime_mut!(LUA)?.push(&opt.id);
		defer! { _ = runtime_mut!(LUA).map(|mut r| r.pop()) }

		let plugin = match LOADER.load_with(&LUA, &opt.id, chunk) {
			Ok(t) => t,
			Err(e) => succ!(warn!("{e}")),
		};
		drop(loader);

		let result = Lives::scope(&self.core, || {
			if let Some(cb) = opt.cb {
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
