use std::time::Duration;

use tokio::sync::oneshot;
use yazi_macro::emit;
use yazi_shared::event::Cmd;

use crate::options::{NotifyLevel, NotifyOpt, PluginOpt};

pub struct AppProxy;

impl AppProxy {
	pub async fn stop() {
		let (tx, rx) = oneshot::channel::<()>();
		emit!(Call(Cmd::new("app:stop").with_any("tx", tx)));
		rx.await.ok();
	}

	pub fn resume() {
		emit!(Call(Cmd::new("app:resume")));
	}

	pub fn notify(opt: NotifyOpt) {
		emit!(Call(Cmd::new("app:notify").with_any("option", opt)));
	}

	pub fn update_notify(dur: Duration) {
		emit!(Call(Cmd::args("app:update_notify", [dur.as_secs_f64()])));
	}

	pub fn notify_warn(title: &str, content: impl ToString) {
		Self::notify(NotifyOpt {
			title:   title.to_owned(),
			content: content.to_string(),
			level:   NotifyLevel::Warn,
			timeout: Duration::from_secs(5),
		});
	}

	pub fn notify_error(title: &str, content: impl ToString) {
		Self::notify(NotifyOpt {
			title:   title.to_owned(),
			content: content.to_string(),
			level:   NotifyLevel::Error,
			timeout: Duration::from_secs(10),
		});
	}

	pub fn plugin(opt: PluginOpt) {
		emit!(Call(Cmd::new("app:plugin").with_any("opt", opt)));
	}

	pub fn plugin_do(opt: PluginOpt) {
		emit!(Call(Cmd::new("app:plugin_do").with_any("opt", opt)));
	}
}
