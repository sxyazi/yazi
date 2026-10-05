use ratatui_core::{buffer::Buffer, layout::Rect, widgets::Widget};
use yazi_core::Core;
use yazi_macro::log_if_err;

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
		log_if_err!("Redrawing `Tasks`", Renderer::new(self.core, "Tasks").render(area, buf));
	}
}
