use std::time::Duration;

use tokio::sync::oneshot;
use yazi_config::MANAGER;
use yazi_macro::emit;
use yazi_shared::{Layer, event::Cmd};

use crate::options::{NotifyLevel, NotifyOpt, PluginOpt};

pub struct AppProxy;

impl AppProxy {
	#[inline]
	pub async fn stop() {
		let (tx, rx) = oneshot::channel::<()>();
		emit!(Call(Cmd::new("stop").with_any("tx", tx), Layer::App));
		rx.await.ok();
	}

	#[inline]
	pub fn resume() {
		emit!(Call(Cmd::new("resume"), Layer::App));
	}

	#[inline]
	pub fn notify(opt: NotifyOpt) {
		if MANAGER._v4_suppress_deprecation_warnings && opt.title.contains("Deprecated") {
			return;
		}

		emit!(Call(Cmd::new("notify").with_any("option", opt), Layer::App));
	}

	#[inline]
	pub fn notify_warn(title: &str, content: impl ToString) {
		Self::notify(NotifyOpt {
			title:   title.to_owned(),
			content: content.to_string(),
			level:   NotifyLevel::Warn,
			timeout: Duration::from_secs(5),
		});
	}

	#[inline]
	pub fn notify_error(title: &str, content: impl ToString) {
		Self::notify(NotifyOpt {
			title:   title.to_owned(),
			content: content.to_string(),
			level:   NotifyLevel::Error,
			timeout: Duration::from_secs(10),
		});
	}

	#[inline]
	pub fn plugin(opt: PluginOpt) {
		emit!(Call(Cmd::new("plugin").with_any("opt", opt), Layer::App));
	}

	#[inline]
	pub fn plugin_do(opt: PluginOpt) {
		emit!(Call(Cmd::new("plugin_do").with_any("opt", opt), Layer::App));
	}
}
