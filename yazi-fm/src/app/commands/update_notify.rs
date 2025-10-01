use anyhow::Result;
use ratatui::layout::Rect;
use yazi_adapter::Dimension;
use yazi_macro::act;
use yazi_parser::notify::TickOpt;
use yazi_shared::data::Data;

use crate::{app::App, notify};

impl App {
	pub(crate) fn update_notify(&mut self, opt: TickOpt) -> Result<Data> {
		let Dimension { rows, columns, .. } = Dimension::available();
		let area =
			notify::Notify::available(Rect { x: 0, y: 0, width: columns, height: rows });

		self.core.notify.tick(opt, area);

		if self.core.notify.messages.is_empty() {
			act!(render, self)
		} else {
			act!(render_partially, self)
		}
	}
}
