use tokio::sync::mpsc;
use yazi_macro::{emit, relay};
use yazi_shared::{Id, SStr, url::UrlBuf};

pub struct AppProxy;

impl AppProxy {
	pub async fn stop() {
		let (tx, mut rx) = mpsc::unbounded_channel();
		emit!(Call(relay!(app:stop).with_replier(tx)));
		rx.recv().await;
	}

	pub async fn resume() {
		let (tx, mut rx) = mpsc::unbounded_channel();
		emit!(Call(relay!(app:resume).with_replier(tx)));
		rx.recv().await;
	}
}

// --- Tasks
pub struct TasksProxy;

impl TasksProxy {
	pub fn update_succeed<I>(id: Id, urls: I, track: bool)
	where
		I: IntoIterator,
		I::Item: Into<UrlBuf>,
	{
		emit!(Call(
			relay!(tasks:update_succeed, [id])
				.with_list("urls", urls.into_iter().map(Into::into))
				.with("track", track)
		));
	}
}

// --- Notify
pub struct NotifyProxy;

impl NotifyProxy {
	pub fn push_warn(title: impl Into<SStr>, content: impl Into<SStr>) {
		emit!(Call(
			relay!(notify:push, [content.into(), title.into()])
				.with("level", SStr::Borrowed("warn"))
				.with("timeout", 5f64)
		));
	}

	pub fn push_error(title: impl Into<SStr>, content: impl Into<SStr>) {
		emit!(Call(
			relay!(notify:push, [content.into(), title.into()])
				.with("level", SStr::Borrowed("error"))
				.with("timeout", 10f64)
		));
	}
}
