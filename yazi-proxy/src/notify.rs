use std::time::Duration;

use yazi_core::notify::MessageOpt;
use yazi_macro::{emit, relay};

pub struct NotifyProxy;

impl NotifyProxy {
	pub fn push(opt: MessageOpt) {
		emit!(Call(relay!(notify:push).with_any("opt", opt)));
	}

	pub fn tick(dur: Duration) {
		emit!(Call(relay!(notify:tick, [dur.as_secs_f64()])));
	}
}
