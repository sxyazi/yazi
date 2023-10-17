use mlua::{AnyUserData, Table};
use shared::RoCell;

use super::Base;
use crate::{layout::{Bar, Gauge, List, Paragraph}, GLOBALS};

pub(super) static COMP_FOLDER: RoCell<Table> = RoCell::new();
pub(super) static COMP_HEADER: RoCell<Table> = RoCell::new();
pub(super) static COMP_MANAGER: RoCell<Table> = RoCell::new();
pub(super) static COMP_STATUS: RoCell<Table> = RoCell::new();

pub fn init() -> mlua::Result<()> {
	COMP_FOLDER.init(GLOBALS.get("Folder")?);
	COMP_HEADER.init(GLOBALS.get("Header")?);
	COMP_MANAGER.init(GLOBALS.get("Manager")?);
	COMP_STATUS.init(GLOBALS.get("Status")?);
	Ok(())
}

pub(super) fn layout(
	values: Vec<AnyUserData>,
	cx: &core::Ctx,
	buf: &mut ratatui::prelude::Buffer,
) -> mlua::Result<()> {
	for value in values {
		if let Ok(c) = value.take::<Paragraph>() {
			c.render(buf)
		} else if let Ok(c) = value.take::<List>() {
			c.render(buf)
		} else if let Ok(c) = value.take::<Bar>() {
			c.render(buf)
		} else if let Ok(c) = value.take::<Base>() {
			c.render(cx, buf)
		} else if let Ok(c) = value.take::<Gauge>() {
			c.render(buf)
		}
	}
	Ok(())
}
