use anyhow::Result;

use crate::app::App;

impl App {
	pub(crate) fn resize(&mut self) -> Result<()> {
		self.cx.manager.active_mut().preview.reset();
		self.render()?;

		self.cx.manager.current_mut().sync_page(true);
		self.cx.manager.hover(None);
		Ok(())
	}
}
