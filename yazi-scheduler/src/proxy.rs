use yazi_macro::{emit, relay};
use yazi_shared::{CompletionToken, Id, SStr, url::UrlBuf};

pub struct AppProxy;

impl AppProxy {
	// FIXME: use replier instead of token
	pub async fn stop() {
		let token = CompletionToken::default();
		emit!(Call(relay!(app:stop).with_any("token", token.clone())));
		token.future().await;
	}

	// FIXME: use replier instead of token
	pub async fn resume() {
		let token = CompletionToken::default();
		emit!(Call(relay!(app:resume).with_any("token", token.clone())));
		token.future().await;
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
		let urls: Vec<_> = urls.into_iter().map(Into::into).collect();
		emit!(Call(relay!(tasks:update_succeed, [id]).with_any("urls", urls).with("track", track)));
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
