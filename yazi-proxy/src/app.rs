use tokio::sync::oneshot;
use yazi_shared::{emit, event::Cmd, Layer};

pub struct App;

impl App {
	pub async fn stop() {
		let (tx, rx) = oneshot::channel::<()>();
		emit!(Call(Cmd::new("stop").with_data(tx), Layer::App));
		rx.await.ok();
	}

	pub fn resume() {
		emit!(Call(Cmd::new("resume"), Layer::App));
	}
}
