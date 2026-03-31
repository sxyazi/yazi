use std::time::Instant;

use anyhow::Result;
use yazi_core::notify::Message;
use yazi_macro::{act, succ};
use yazi_parser::{notify::PushForm, spark::SparkKind};
use yazi_shared::{Source, data::Data};

use crate::{Actor, Ctx};

pub struct Push;

impl Actor for Push {
	type Form = PushForm;

	const NAME: &str = "push";

	fn act(cx: &mut Ctx, form: Self::Form) -> Result<Data> {
		let instant = Instant::now();

		let mut msg = Message::from(form.opt);
		msg.timeout += instant - cx.notify.messages.first().map_or(instant, |m| m.instant);

		if cx.notify.messages.iter().all(|m| m != &msg) {
			cx.notify.messages.push(msg);
			act!(notify:tick, cx)?;
		}
		succ!();
	}

	fn hook(cx: &Ctx, _: &Self::Form) -> Option<SparkKind> {
		match cx.source() {
			Source::Relay => Some(SparkKind::RelayNotifyPush),
			_ => None,
		}
	}
}
