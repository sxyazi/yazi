use std::time::Duration;

use yazi_macro::{emit, relay};
use yazi_parser::notify::{PushLevel, PushOpt};

pub struct NotifyProxy;

impl NotifyProxy {
	pub fn push(opt: PushOpt) {
		emit!(Call(relay!(notify:push).with_any("opt", opt)));
	}

	pub fn push_warn(title: impl Into<String>, content: impl Into<String>) {
		Self::push(PushOpt {
			title:   title.into(),
			content: content.into(),
			level:   PushLevel::Warn,
			timeout: Duration::from_secs(5),
		});
	}

	pub fn push_error(title: &str, content: impl ToString) {
		Self::push(PushOpt {
			title:   title.to_owned(),
			content: content.to_string(),
			level:   PushLevel::Error,
			timeout: Duration::from_secs(10),
		});
	}

	pub fn tick(dur: Duration) {
		emit!(Call(relay!(notify:tick, [dur.as_secs_f64()])));
	}
}
