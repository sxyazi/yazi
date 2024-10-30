use std::sync::Arc;

use mlua::Value;
use ratatui::layout::Position;
use tracing::error;
use yazi_config::LAYOUT;
use yazi_macro::render;
use yazi_shared::event::Cmd;

use crate::{Root, app::App, lives::Lives};

struct Opt;

impl From<Cmd> for Opt {
	fn from(_: Cmd) -> Self { Self }
}

impl From<()> for Opt {
	fn from(_: ()) -> Self { Self }
}

impl App {
	#[yazi_codegen::command]
	pub fn reflow(&mut self, _: Opt) {
		let Some(size) = self.term.as_ref().and_then(|t| t.size().ok()) else { return };
		let mut layout = *LAYOUT.load_full();

		let result = Lives::scope(&self.cx, |_| {
			let comps = Root::reflow((Position::ORIGIN, size).into())?;

			for v in comps.sequence_values::<Value>() {
				let Value::Table(t) = v? else {
					error!("`reflow()` must return a table of components");
					continue;
				};

				let id: mlua::String = t.get("_id")?;
				match id.to_str()? {
					"current" => layout.current = *t.raw_get::<_, yazi_plugin::elements::Rect>("_area")?,
					"preview" => layout.preview = *t.raw_get::<_, yazi_plugin::elements::Rect>("_area")?,
					_ => {}
				}
			}
			Ok(())
		});

		if layout != *LAYOUT.load_full() {
			LAYOUT.store(Arc::new(layout));
			render!();
		}

		if let Err(e) = result {
			error!("Failed to `reflow()` the `Root` component:\n{e}");
		}
	}
}
