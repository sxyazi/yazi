use anyhow::Result;
use mlua::Value;
use ratatui::layout::Position;
use tracing::error;
use yazi_actor::lives::Lives;
use yazi_config::LAYOUT;
use yazi_macro::{render, succ};
use yazi_parser::VoidOpt;
use yazi_shared::data::Data;

use crate::{Root, app::App};

impl App {
	pub fn reflow(&mut self, _: VoidOpt) -> Result<Data> {
		let Some(size) = self.term.as_ref().and_then(|t| t.size().ok()) else { succ!() };
		let mut layout = LAYOUT.get();

		let result = Lives::scope(&self.core, || {
			let comps = Root::reflow((Position::ORIGIN, size).into())?;

			for v in comps.sequence_values::<Value>() {
				let Value::Table(t) = v? else {
					error!("`reflow()` must return a table of components");
					continue;
				};

				let id: mlua::String = t.get("_id")?;
				match &*id.as_bytes() {
					b"current" => layout.current = *t.raw_get::<yazi_binding::elements::Rect>("_area")?,
					b"preview" => layout.preview = *t.raw_get::<yazi_binding::elements::Rect>("_area")?,
					b"progress" => layout.progress = *t.raw_get::<yazi_binding::elements::Rect>("_area")?,
					_ => {}
				}
			}
			Ok(())
		});

		if layout != LAYOUT.get() {
			LAYOUT.set(layout);
			render!();
		}

		if let Err(ref e) = result {
			error!("Failed to `reflow()` the `Root` component:\n{e}");
		}
		succ!();
	}
}
