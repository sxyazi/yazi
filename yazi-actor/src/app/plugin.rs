use anyhow::Result;
use yazi_core::{Ctx, app::PluginMode};
use yazi_macro::succ;
use yazi_parser::app::PluginForm;
use yazi_proxy::AppProxy;
use yazi_runner::loader::LOADER;
use yazi_scheduler::NotifyProxy;
use yazi_shared::data::Data;

use crate::{Actor, act};

pub struct Plugin;

impl Actor for Plugin {
	type Form = PluginForm;

	const NAME: &str = "plugin";

	fn act(cx: &mut Ctx, Self::Form { opt, scope }: Self::Form) -> Result<Data> {
		let mode = LOADER.read().get(&*opt.name).map(|chunk| opt.effective_mode(chunk));

		if mode.unwrap_or(opt.mode) == PluginMode::Async {
			succ!(cx.core.tasks.scheduler.plugin_entry(opt.into()));
		} else if mode == Some(PluginMode::Sync) {
			return act!(app:plugin_do, cx, opt);
		}

		tokio::spawn(async move {
			match LOADER.ensure(&opt.name, |_| ()).await {
				Ok(()) => AppProxy::plugin_do(opt, scope),
				Err(e) => NotifyProxy::push_error("Plugin load failed", e.to_string()),
			}
		});
		succ!();
	}
}
