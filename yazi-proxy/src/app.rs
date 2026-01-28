use yazi_macro::{emit, relay};
use yazi_parser::app::{PluginOpt, QuitOpt, TaskSummary};
use yazi_shared::CompletionToken;

pub struct AppProxy;

impl AppProxy {
	pub fn quit(opt: QuitOpt) {
		emit!(Call(relay!(app:quit).with_any("opt", opt)));
	}

	pub async fn stop() {
		let token = CompletionToken::default();
		emit!(Call(relay!(app:stop).with_any("token", token.clone())));
		token.future().await;
	}

	pub async fn resume() {
		let token = CompletionToken::default();
		emit!(Call(relay!(app:resume).with_any("token", token.clone())));
		token.future().await;
	}

	pub fn plugin(opt: PluginOpt) {
		emit!(Call(relay!(app:plugin).with_any("opt", opt)));
	}

	pub fn plugin_do(opt: PluginOpt) {
		emit!(Call(relay!(app:plugin_do).with_any("opt", opt)));
	}

	pub fn update_progress(summary: TaskSummary) {
		emit!(Call(relay!(app:update_progress).with_any("summary", summary)));
	}
}
