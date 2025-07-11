use mlua::{ObjectLike, Table};
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
use tracing::error;
use yazi_binding::elements::render_once;
use yazi_core::Core;
use yazi_plugin::LUA;

pub(crate) struct Modal<'a> {
	core: &'a Core,
}

impl<'a> Modal<'a> {
	#[inline]
	pub(crate) fn new(core: &'a Core) -> Self { Self { core } }
}

impl Widget for Modal<'_> {
	fn render(self, area: Rect, buf: &mut Buffer) {
		let mut f = || {
			let area = yazi_binding::elements::Rect::from(area);
			let root = LUA.globals().raw_get::<Table>("Modal")?.call_method::<Table>("new", area)?;

			render_once(root.call_method("children_redraw", ())?, buf, |p| self.core.mgr.area(p));
			Ok::<_, mlua::Error>(())
		};
		if let Err(e) = f() {
			error!("Failed to redraw the `Modal` component:\n{e}");
		}
	}
}
