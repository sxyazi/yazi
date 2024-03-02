use tokio::sync::oneshot;
use yazi_shared::{emit, event::Cmd, Layer};

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

	#[inline]
	pub fn warn(title: &str, content: &str) {
		emit!(Call(
			Cmd::new("notify")
				.with("title", title)
				.with("content", content)
				.with("level", "warn")
				.with("timeout", 5),
			Layer::App
		));
	}
}
