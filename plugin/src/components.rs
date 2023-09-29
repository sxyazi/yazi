use core::Ctx;

use mlua::{Result, Table, TableExt};
use ratatui::layout;

use crate::{bindings, layout::{Paragraph, Rect}, GLOBALS, LUA};

pub struct Status;

impl Status {
	pub fn render(cx: &Ctx, area: layout::Rect) -> Result<Vec<Paragraph>> {
		LUA.scope(|scope| {
			let tbl = LUA.create_table()?;
			tbl.set("manager", bindings::Manager::make(scope, &cx.manager)?)?;
			tbl.set("tasks", bindings::Tasks::make(scope, &cx.tasks)?)?;
			GLOBALS.set("cx", tbl)?;

			let comp: Table = GLOBALS.get("Status")?;
			comp.call_method::<_, _>("render", Rect(area))
		})
	}
}

pub struct Folder {
	pub kind: u8,
}

impl Folder {
	fn args(&self) -> Result<Table> {
		let tbl = LUA.create_table()?;
		tbl.set("kind", self.kind)?;
		Ok(tbl)
	}

	pub fn render(self, cx: &Ctx, area: layout::Rect) -> Result<Vec<Paragraph>> {
		LUA.scope(|scope| {
			let tbl = LUA.create_table()?;
			tbl.set("manager", bindings::Manager::make(scope, &cx.manager)?)?;
			GLOBALS.set("cx", tbl)?;

			let comp: Table = GLOBALS.get("Folder")?;
			comp.call_method::<_, _>("render", (Rect(area), self.args()?))
		})
	}
}
