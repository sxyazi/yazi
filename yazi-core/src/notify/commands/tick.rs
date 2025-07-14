use std::time::Duration;

use ratatui::layout::Rect;
use yazi_parser::notify::TickOpt;
use yazi_proxy::AppProxy;

use crate::notify::Notify;

impl Notify {
	pub fn tick(&mut self, opt: TickOpt, area: Rect) {
		self.tick_handle.take().map(|h| h.abort());

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
			AppProxy::update_notify(interval);
		}));
	}
}
