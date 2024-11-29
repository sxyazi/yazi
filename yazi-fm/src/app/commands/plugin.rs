use std::fmt::Display;

use mlua::ObjectLike;
use scopeguard::defer;
use tracing::warn;
use yazi_dds::Sendable;
use yazi_plugin::{LUA, RtRef, loader::LOADER};
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
			if LOADER.ensure(&opt.id).await.is_ok() {
				AppProxy::plugin_do(opt);
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

		if opt.mode.auto_then(chunk.sync_entry) != PluginMode::Sync {
			return self.cx.tasks.plugin_micro(opt);
		}

		match LUA.named_registry_value::<RtRef>("rt") {
			Ok(mut r) => r.push(&opt.id),
			Err(e) => return warn!("{e}"),
		}
		defer! { _ = LUA.named_registry_value::<RtRef>("rt").map(|mut r| r.pop()) }

		let plugin = match LOADER.load_with(&LUA, &opt.id, chunk) {
			Ok(plugin) => plugin,
			Err(e) => return warn!("{e}"),
		};
		drop(loader);

		_ = Lives::scope(&self.cx, || {
			if let Some(cb) = opt.cb {
				cb(&LUA, plugin)
			} else {
				let job = LUA.create_table_from([("args", Sendable::dict_to_table(&LUA, opt.args)?)])?;

				// TODO: remove this
				yazi_plugin::isolate::install_entry_warn(&LUA, &job, opt._old_args).ok();

				plugin.call_method("entry", job)
			}
		});
	}
}
