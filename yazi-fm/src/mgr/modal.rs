use ratatui_core::{buffer::Buffer, layout::Rect, widgets::Widget};
use tracing::error;
use yazi_core::Core;

use crate::Renderer;

pub(crate) struct Modal<'a> {
	core: &'a mut Core,
}

impl<'a> Modal<'a> {
	#[inline]
	pub(crate) fn new(core: &'a mut Core) -> Self { Self { core } }
}

impl Widget for Modal<'_> {
	fn render(self, area: Rect, buf: &mut Buffer) {
		let result =
			Renderer::new(self.core, "Modal").with_redrawer("children_redraw").render(area, buf);

		if let Err(e) = result {
			error!("Failed to redraw the `Modal` component:\n{e}");
		}
	}
}
