use mlua::TableExt;
use ratatui::widgets::Widget;
use tracing::error;

use super::{layout, COMP_MANAGER};
use crate::layout::Rect;

pub struct Manager<'a> {
	cx: &'a yazi_core::Ctx,
}

impl<'a> Manager<'a> {
	pub fn new(cx: &'a yazi_core::Ctx) -> Self { Self { cx } }
}

impl<'a> Widget for Manager<'a> {
	fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
		let mut f = || layout(COMP_MANAGER.call_method::<_, _>("render", Rect(area))?, self.cx, buf);
		if let Err(e) = f() {
			error!("{:?}", e);
		}
	}
}
