use ratatui_core::{buffer::Buffer, layout::Rect, widgets::Widget};
use yazi_core::Core;
use yazi_macro::log_if_err;

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
		log_if_err!(
			"Redrawing `Modal`",
			Renderer::new(self.core, "Modal").with_redrawer("children_redraw").render(area, buf)
		);
	}
}
