use std::time::Duration;

use ratatui::layout::Rect;
use yazi_macro::emit;
use yazi_shared::event::{Cmd, CmdCow, Data};

use crate::notify::Notify;

pub struct Opt {
	interval: Duration,
}

impl TryFrom<CmdCow> for Opt {
	type Error = ();

	fn try_from(c: CmdCow) -> Result<Self, Self::Error> {
		let interval = c.first().and_then(Data::as_f64).ok_or(())?;
		if interval < 0.0 {
			return Err(());
		}

		Ok(Self { interval: Duration::from_secs_f64(interval) })
	}
}

impl TryFrom<Cmd> for Opt {
	type Error = ();

	fn try_from(c: Cmd) -> Result<Self, Self::Error> { Self::try_from(CmdCow::from(c)) }
}

impl Notify {
	pub fn tick(&mut self, opt: impl TryInto<Opt>, area: Rect) {
		self.tick_handle.take().map(|h| h.abort());
		let Ok(opt) = opt.try_into() else {
			return;
		};

		let limit = self.limit(area);
		if limit == 0 {
			return;
		}

		for m in &mut self.messages[..limit] {
			if m.timeout.is_zero() {
				m.percent = m.percent.saturating_sub(20);
			} else if m.percent < 100 {
				m.percent += 20;
			} else {
				m.timeout = m.timeout.saturating_sub(opt.interval);
			}
		}

		self.messages.retain(|m| m.percent > 0 || !m.timeout.is_zero());
		let limit = self.limit(area);
		let timeouts: Vec<_> = self.messages[..limit]
			.iter()
			.filter(|&m| m.percent == 100 && !m.timeout.is_zero())
			.map(|m| m.timeout)
			.collect();

		let interval = if timeouts.len() != limit {
			Duration::from_millis(50)
		} else if let Some(min) = timeouts.iter().min() {
			*min
		} else {
			return;
		};

		self.tick_handle = Some(tokio::spawn(async move {
			tokio::time::sleep(interval).await;
			emit!(Call(Cmd::args("app:update_notify", &[interval.as_secs_f64()])));
		}));
	}
}
