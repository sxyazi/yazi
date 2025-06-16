use std::fmt::Display;

use mlua::ObjectLike;
use scopeguard::defer;
use tracing::{error, warn};
use yazi_dds::Sendable;
use yazi_plugin::{LUA, loader::{LOADER, Loader}, runtime_mut};
use yazi_proxy::{AppProxy, options::{PluginMode, PluginOpt}};

use crate::{app::App, lives::Lives};

impl App {
	pub(crate) fn plugin(&mut self, opt: impl TryInto<PluginOpt, Error = impl Display>) {
		let mut opt = match opt.try_into() {
			Ok(opt) => opt as PluginOpt,
			Err(e) => return warn!("{e}"),
		};

		let mut hits = false;
		if let Some(chunk) = LOADER.read().get(opt.id.as_ref()) {
			hits = true;
			opt.mode = opt.mode.auto_then(chunk.sync_entry);
		}

		if opt.mode == PluginMode::Async {
			return self.cx.tasks.plugin_micro(opt);
		} else if opt.mode == PluginMode::Sync && hits {
			return self.plugin_do(opt);
		}

		tokio::spawn(async move {
			match LOADER.ensure(&opt.id, |_| ()).await {
				Ok(()) => AppProxy::plugin_do(opt),
				Err(e) => AppProxy::notify_error("Plugin load failed", e),
			}
		});
	}

	pub(crate) fn plugin_do(&mut self, opt: impl TryInto<PluginOpt, Error = impl Display>) {
		let opt = match opt.try_into() {
			Ok(opt) => opt as PluginOpt,
			Err(e) => return warn!("{e}"),
		};

		let loader = LOADER.read();
		let Some(chunk) = loader.get(opt.id.as_ref()) else {
			return warn!("plugin `{}` not found", opt.id);
		};

		if let Err(e) = Loader::compatible_or_error(&opt.id, chunk) {
			return AppProxy::notify_error("Incompatible plugin", e);
		}

		if opt.mode.auto_then(chunk.sync_entry) != PluginMode::Sync {
			return self.cx.tasks.plugin_micro(opt);
		}

		match runtime_mut!(LUA) {
			Ok(mut r) => r.push(&opt.id),
			Err(e) => return warn!("{e}"),
		}
		defer! { _ = runtime_mut!(LUA).map(|mut r| r.pop()) }

		let plugin = match LOADER.load_with(&LUA, &opt.id, chunk) {
			Ok(t) => t,
			Err(e) => return warn!("{e}"),
		};
		drop(loader);

		let result = Lives::scope(&self.cx, || {
			if let Some(cb) = opt.cb {
				cb(&LUA, plugin)
			} else {
				let job = LUA.create_table_from([("args", Sendable::args_to_table(&LUA, opt.args)?)])?;
				plugin.call_method("entry", job)
			}
		});
		if let Err(e) = result {
			error!("Sync plugin `{}` failed: {e}", opt.id);
		}
	}
}
