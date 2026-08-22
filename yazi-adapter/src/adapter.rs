use std::{fmt::{self, Debug}, path::PathBuf, sync::OnceLock};

use anyhow::Result;
use ratatui_core::layout::Rect;
use yazi_emulator::EMULATOR;
use yazi_shim::cell::SyncCell;
use yazi_widgets::clear::ClearInventory;

use crate::{ADAPTOR, drivers::{Driver, Drivers}};

#[derive(Default)]
pub struct Adapter {
	driver:        OnceLock<Driver>,
	shown:         SyncCell<Option<Rect>>,
	pub collision: SyncCell<bool>,
}

impl Debug for Adapter {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self.driver.get() {
			Some(driver) => driver.fmt(f),
			None => f.write_str("Pending"),
		}
	}
}

impl Adapter {
	pub async fn image_show<P>(&self, path: P, max: Rect) -> Result<Rect>
	where
		P: Into<PathBuf>,
	{
		let probe = &EMULATOR.probe;
		probe.wait(probe.id.get()).await;

		let driver = self.driver.get_or_init(|| {
			let driver = Drivers::matches(&EMULATOR);
			driver.start();
			driver
		});
		driver.image_show(path, max).await
	}

	pub fn image_hide(&self) -> Result<()> {
		let Some(area) = self.shown.replace(None) else { return Ok(()) };
		match self.driver.get() {
			Some(driver) => driver.image_erase(area),
			None => Ok(()),
		}
	}

	pub fn shown_area(&self) -> Option<Rect> { self.shown.get() }

	pub(super) fn shown_store(&self, area: Rect) { self.shown.set(Some(area)); }
}

inventory::submit! {
	ClearInventory {
		clear: |area| {
			let overlap = area.intersection(ADAPTOR.shown.get()?);
			if overlap.area() == 0 {
				return None;
			}

			ADAPTOR.driver.get()?.image_erase(overlap).ok();
			ADAPTOR.collision.set(true);
			Some(overlap)
		},
	}
}
