use ratatui_core::{buffer::Buffer, layout::Rect, widgets::Widget};
use yazi_config::LAYOUT;
use yazi_core::Core;
use yazi_macro::log_if_err;

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
		log_if_err!(
			"Redrawing `Progress`",
			Renderer::new(self.core, "Progress").with_constructor("use").render(area, buf)
		);
	}
}
