use mlua::TableExt;
use ratatui::widgets::Widget;
use tracing::error;

use super::{layout, COMP_FOLDER};
use crate::{layout::Rect, LUA};

pub struct Folder<'a> {
	cx:   &'a core::Ctx,
	kind: u8,
}

impl<'a> Folder<'a> {
	#[inline]
	pub fn parent(cx: &'a core::Ctx) -> Self { Self { cx, kind: 0 } }

	#[inline]
	pub fn current(cx: &'a core::Ctx) -> Self { Self { cx, kind: 1 } }

	#[inline]
	pub fn preview(cx: &'a core::Ctx) -> Self { Self { cx, kind: 2 } }
}

impl<'a> Widget for Folder<'a> {
	fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
		let mut f = || {
			let args = LUA.create_table()?;
			args.set("kind", self.kind)?;
			layout(COMP_FOLDER.call_method::<_, _>("render", (Rect(area), args))?, self.cx, buf)
		};
		if let Err(e) = f() {
			error!("{:?}", e);
		}
	}
}
