use mlua::TableExt;
use ratatui::widgets::Widget;
use tracing::error;

use super::{layout, COMP_HEADER};
use crate::layout::Rect;

pub struct Header<'a> {
	cx: &'a yazi_core::Ctx,
}

impl<'a> Header<'a> {
	#[inline]
	pub fn new(cx: &'a yazi_core::Ctx) -> Self { Self { cx } }
}

impl<'a> Widget for Header<'a> {
	fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
		let mut f = || layout(COMP_HEADER.call_method::<_, _>("render", Rect(area))?, self.cx, buf);
		if let Err(e) = f() {
			error!("{:?}", e);
		}
	}
}
