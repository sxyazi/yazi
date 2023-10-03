use mlua::{Result, Table, TableExt};
use ratatui::layout;

use crate::{layout::{Paragraph, Rect}, GLOBALS, LUA};

pub struct Status;

impl Status {
	pub fn render(area: layout::Rect) -> Result<Vec<Paragraph>> {
		let comp: Table = GLOBALS.get("Status")?;
		comp.call_method::<_, _>("render", Rect(area))
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

	pub fn render(self, area: layout::Rect) -> Result<Vec<Paragraph>> {
		let comp: Table = GLOBALS.get("Folder")?;
		comp.call_method::<_, _>("render", (Rect(area), self.args()?))
	}
}
