use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
use tracing::error;
use yazi_config::LAYOUT;
use yazi_core::Core;

use crate::Renderer;

pub(crate) struct Progress<'a> {
	core: &'a mut Core,
}

impl<'a> Progress<'a> {
	pub(crate) fn new(core: &'a mut Core) -> Self { Self { core } }
}

impl Widget for Progress<'_> {
	fn render(self, _: Rect, buf: &mut Buffer) {
		let area = LAYOUT.get().progress;
		let result = Renderer::new(self.core, "Progress").with_constructor("use").render(area, buf);

		if let Err(e) = result {
			error!("Failed to redraw the `Progress` component:\n{e}");
		}
	}
}
