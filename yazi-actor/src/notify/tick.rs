use std::time::Duration;

use anyhow::Result;
use ratatui::layout::Rect;
use yazi_core::notify::Notify;
use yazi_emulator::Dimension;
use yazi_macro::succ;
use yazi_parser::notify::TickOpt;
use yazi_proxy::NotifyProxy;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct Tick;

impl Actor for Tick {
	type Options = TickOpt;

	const NAME: &str = "tick";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		cx.notify.tick_handle.take().map(|h| h.abort());

		let Dimension { rows, columns, .. } = Dimension::available();
		let area = Notify::available(Rect { x: 0, y: 0, width: columns, height: rows });

		let limit = cx.notify.limit(area);
		if limit == 0 {
			succ!();
		}

		for m in &mut cx.notify.messages[..limit] {
			if m.timeout.is_zero() {
				m.percent = m.percent.saturating_sub(20);
			} else if m.percent < 100 {
				m.percent += 20;
			} else {
				m.timeout = m.timeout.saturating_sub(opt.interval);
			}
		}

		cx.notify.messages.retain(|m| m.percent > 0 || !m.timeout.is_zero());
		let limit = cx.notify.limit(area);
		let timeouts: Vec<_> = cx.notify.messages[..limit]
			.iter()
			.filter(|&m| m.percent == 100 && !m.timeout.is_zero())
			.map(|m| m.timeout)
			.collect();

		let interval = if timeouts.len() != limit {
			Duration::from_millis(50)
		} else if let Some(min) = timeouts.iter().min() {
			*min
		} else {
			succ!();
		};

		cx.notify.tick_handle = Some(tokio::spawn(async move {
			tokio::time::sleep(interval).await;
			NotifyProxy::tick(interval);
		}));
		succ!();
	}
}
