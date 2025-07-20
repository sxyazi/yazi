use std::time::Duration;

use tokio::sync::oneshot;
use yazi_macro::{emit, relay};
use yazi_parser::app::{NotifyLevel, NotifyOpt, PluginOpt, TasksProgress};

pub struct AppProxy;

impl AppProxy {
	pub async fn stop() {
		let (tx, rx) = oneshot::channel::<()>();
		emit!(Call(relay!(app:stop).with_any("tx", tx)));
		rx.await.ok();
	}

	pub fn resume() {
		emit!(Call(relay!(app:resume)));
	}

	pub fn notify(opt: NotifyOpt) {
		emit!(Call(relay!(app:notify).with_any("option", opt)));
	}

	pub fn update_notify(dur: Duration) {
		emit!(Call(relay!(app:update_notify, [dur.as_secs_f64()])));
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
		emit!(Call(relay!(app:plugin).with_any("opt", opt)));
	}

	pub fn plugin_do(opt: PluginOpt) {
		emit!(Call(relay!(app:plugin_do).with_any("opt", opt)));
	}

	pub fn update_progress(progress: TasksProgress) {
		emit!(Call(relay!(app:update_progress).with_any("progress", progress)));
	}
}
