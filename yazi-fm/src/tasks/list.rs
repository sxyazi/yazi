use ratatui_core::{buffer::Buffer, layout::Rect, widgets::Widget};
use tracing::error;
use yazi_core::Core;

use crate::Renderer;

pub(crate) struct List<'a> {
	core: &'a mut Core,
}

impl<'a> List<'a> {
	#[inline]
	pub(crate) fn new(core: &'a mut Core) -> Self { Self { core } }
}

impl Widget for List<'_> {
	fn render(self, area: Rect, buf: &mut Buffer) {
		if let Err(e) = Renderer::new(self.core, "Tasks").render(area, buf) {
			error!("Failed to redraw the `Tasks` component:\n{e}");
		}
	}
}
