use mlua::{ObjectLike, Table};
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
use tracing::error;
use yazi_config::LAYOUT;
use yazi_plugin::{LUA, elements::render_once};

use crate::Ctx;

pub(crate) struct Progress<'a> {
	cx: &'a Ctx,
}

impl<'a> Progress<'a> {
	pub(crate) fn new(cx: &'a Ctx) -> Self { Self { cx } }
}

impl Widget for Progress<'_> {
	fn render(self, _: Rect, buf: &mut Buffer) {
		let mut f = || {
			let area = yazi_plugin::elements::Rect::from(LAYOUT.get().progress);
			let progress =
				LUA.globals().raw_get::<Table>("Progress")?.call_method::<Table>("use", area)?;

			render_once(progress.call_method("redraw", ())?, buf, |p| self.cx.mgr.area(p));
			Ok::<_, mlua::Error>(())
		};
		if let Err(e) = f() {
			error!("Failed to redraw the `Progress` component:\n{e}");
		}
	}
}
