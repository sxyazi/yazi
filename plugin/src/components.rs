use mlua::{AnyUserData, Table, TableExt};

use crate::{layout::{List, Paragraph, Rect}, GLOBALS, LUA};

#[inline]
fn layout(values: Vec<AnyUserData>, buf: &mut ratatui::prelude::Buffer) -> mlua::Result<()> {
	for value in values {
		if let Ok(c) = value.take::<Paragraph>() {
			c.render(buf)
		} else if let Ok(c) = value.take::<List>() {
			c.render(buf)
		}
	}
	Ok(())
}

// --- Status
pub struct Status;

impl Status {
	pub fn render(
		self,
		area: ratatui::layout::Rect,
		buf: &mut ratatui::prelude::Buffer,
	) -> mlua::Result<()> {
		let comp: Table = GLOBALS.get("Status")?;
		let values: Vec<AnyUserData> = comp.call_method::<_, _>("render", Rect(area))?;

		layout(values, buf)
	}
}

// --- Folder
pub struct Folder {
	pub kind: u8,
}

impl Folder {
	fn args(&self) -> mlua::Result<Table> {
		let tbl = LUA.create_table()?;
		tbl.set("kind", self.kind)?;
		Ok(tbl)
	}

	pub fn render(
		self,
		area: ratatui::layout::Rect,
		buf: &mut ratatui::prelude::Buffer,
	) -> mlua::Result<()> {
		let comp: Table = GLOBALS.get("Folder")?;
		let values: Vec<AnyUserData> =
			comp.call_method::<_, _>("render", (Rect(area), self.args()?))?;

		layout(values, buf)
	}
}
