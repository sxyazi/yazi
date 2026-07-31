use std::{fmt::{self, Debug}, path::PathBuf};

use anyhow::Result;
use ratatui_core::layout::Rect;
use tokio::time::{Duration, timeout};
use yazi_emulator::{EMULATOR, Emulator};
use yazi_shared::CompletionCell;
use yazi_shim::cell::SyncCell;
use yazi_widgets::clear::ClearInventory;

use crate::{ADAPTOR, drivers::{Driver, Drivers}};

#[derive(Default)]
pub struct Adapter {
	driver:        CompletionCell<Driver>,
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
		let driver = match timeout(Duration::from_secs(10), self.driver.wait()).await {
			Ok(driver) => *driver,
			Err(_) => self.resolve(&EMULATOR.load()),
		};
		driver.image_show(path, max).await
	}

	pub fn image_hide(&self) -> Result<()> {
		let Some(area) = self.shown.replace(None) else { return Ok(()) };
		match self.driver.get() {
			Some(driver) => driver.image_erase(area),
			None => Ok(()),
		}
	}

	pub(super) fn shown_store(&self, area: Rect) { self.shown.set(Some(area)); }

	pub fn resolve(&self, emulator: &Emulator) -> Driver {
		*self.driver.get_or_init(|| {
			let driver = Drivers::matches(emulator);
			driver.start();
			driver
		})
	}
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
