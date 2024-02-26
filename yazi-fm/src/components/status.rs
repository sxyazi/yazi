use mlua::{Table, TableExt};
use ratatui::widgets::Widget;
use tracing::error;
use yazi_plugin::{bindings::Cast, elements::{render_widgets, Rect}, LUA};

pub(crate) struct Status;

impl Widget for Status {
	fn render(self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
		let mut f = || {
			let area = Rect::cast(&LUA, area)?;
			let comp: Table = LUA.globals().raw_get("Status")?;
			render_widgets(comp.call_method("render", area)?, buf);
			Ok::<_, anyhow::Error>(())
		};
		if let Err(e) = f() {
			error!("{:?}", e);
		}
	}
}
