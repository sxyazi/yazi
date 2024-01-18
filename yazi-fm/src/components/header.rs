use mlua::{Table, TableExt};
use ratatui::{buffer::Buffer, widgets::Widget};
use tracing::error;
use yazi_plugin::{bindings::Cast, elements::{render_widgets, Rect}, LUA};

pub(crate) struct Header;

impl Widget for Header {
	fn render(self, area: ratatui::layout::Rect, buf: &mut Buffer) {
		let mut f = || {
			let area = Rect::cast(&LUA, area)?;
			let comp: Table = LUA.globals().get("Header")?;
			render_widgets(comp.call_method("render", area)?, buf);
			Ok::<_, anyhow::Error>(())
		};
		if let Err(e) = f() {
			error!("{:?}", e);
		}
	}
}
