use mlua::{ObjectLike, Table};
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
use tracing::error;
use yazi_binding::elements::render_once;
use yazi_config::LAYOUT;
use yazi_core::Core;
use yazi_plugin::LUA;

pub(crate) struct Progress<'a> {
	core: &'a Core,
}

impl<'a> Progress<'a> {
	pub(crate) fn new(core: &'a Core) -> Self { Self { core } }
}

impl Widget for Progress<'_> {
	fn render(self, _: Rect, buf: &mut Buffer) {
		let mut f = || {
			let area = yazi_binding::elements::Rect::from(LAYOUT.get().progress);
			let progress =
				LUA.globals().raw_get::<Table>("Progress")?.call_method::<Table>("use", area)?;

			render_once(progress.call_method("redraw", ())?, buf, |p| self.core.mgr.area(p));
			Ok::<_, mlua::Error>(())
		};
		if let Err(e) = f() {
			error!("Failed to redraw the `Progress` component:\n{e}");
		}
	}
}
