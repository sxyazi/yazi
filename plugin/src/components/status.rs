use mlua::TableExt;
use ratatui::widgets::Widget;
use tracing::error;

use super::{layout, COMP_STATUS};
use crate::layout::Rect;

pub struct Status<'a> {
	cx: &'a core::Ctx,
}

impl<'a> Status<'a> {
	#[inline]
	pub fn new(cx: &'a core::Ctx) -> Self { Self { cx } }
}

impl<'a> Widget for Status<'a> {
	fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
		let mut f = || layout(COMP_STATUS.call_method::<_, _>("render", Rect(area))?, self.cx, buf);
		if let Err(e) = f() {
			error!("{:?}", e);
		}
	}
}
