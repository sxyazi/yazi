use std::time::{Duration, Instant};

use anyhow::Result;
use yazi_core::notify::Message;
use yazi_dds::spark::SparkKind;
use yazi_macro::succ;
use yazi_parser::notify::PushOpt;
use yazi_proxy::NotifyProxy;
use yazi_shared::{Source, data::Data};

use crate::{Actor, Ctx};

pub struct Push;

impl Actor for Push {
	type Options = PushOpt;

	const NAME: &str = "push";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		let instant = Instant::now();

		let mut msg = Message::from(opt);
		msg.timeout += instant - cx.notify.messages.first().map_or(instant, |m| m.instant);

		if cx.notify.messages.iter().all(|m| m != &msg) {
			cx.notify.messages.push(msg);
			NotifyProxy::tick(Duration::ZERO);
		}
		succ!();
	}

	fn hook(cx: &Ctx, _: &Self::Options) -> Option<SparkKind> {
		match cx.source() {
			Source::Relay => Some(SparkKind::RelayNotifyPush),
			_ => None,
		}
	}
}
