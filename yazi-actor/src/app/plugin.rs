use anyhow::Result;
use yazi_core::app::PluginMode;
use yazi_macro::{act, succ};
use yazi_parser::app::PluginForm;
use yazi_proxy::AppProxy;
use yazi_runner::loader::LOADER;
use yazi_scheduler::NotifyProxy;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct Plugin;

impl Actor for Plugin {
	type Form = PluginForm;

	const NAME: &str = "plugin";

	fn act(cx: &mut Ctx, Self::Form { mut opt }: Self::Form) -> Result<Data> {
		let mut hits = false;
		if let Some(chunk) = LOADER.read().get(&*opt.id) {
			hits = true;
			opt.mode = opt.mode.auto_then(chunk.sync_entry);
		}

		if opt.mode == PluginMode::Async {
			succ!(cx.core.tasks.scheduler.plugin_entry(opt.into()));
		} else if opt.mode == PluginMode::Sync && hits {
			return act!(app:plugin_do, cx, opt);
		}

		tokio::spawn(async move {
			match LOADER.ensure(&opt.id, |_| ()).await {
				Ok(()) => AppProxy::plugin_do(opt),
				Err(e) => NotifyProxy::push_error("Plugin load failed", e.to_string()),
			}
		});
		succ!();
	}
}
