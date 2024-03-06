use std::time::Duration;

use tokio::sync::oneshot;
use yazi_shared::{emit, event::Cmd, Layer};

use crate::options::{NotifyLevel, NotifyOpt};

pub struct AppProxy;

impl AppProxy {
	#[inline]
	pub async fn stop() {
		let (tx, rx) = oneshot::channel::<()>();
		emit!(Call(Cmd::new("stop").with_data(tx), Layer::App));
		rx.await.ok();
	}

	#[inline]
	pub fn resume() {
		emit!(Call(Cmd::new("resume"), Layer::App));
	}

	pub fn notify(opt: NotifyOpt) {
		emit!(Call(Cmd::new("notify").with_data(opt), Layer::App));
	}

	#[inline]
	pub fn notify_warn(title: &str, content: &str) {
		emit!(Call(
			Cmd::new("notify").with_data(NotifyOpt {
				title:   title.to_owned(),
				content: content.to_owned(),
				level:   NotifyLevel::Warn,
				timeout: Duration::from_secs(5),
			}),
			Layer::App
		));
	}
}
