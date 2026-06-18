use std::{fmt::{self, Debug}, ops::Deref};

use anyhow::Result;
use ratatui_core::layout::Rect;
use yazi_emulator::EMULATOR;
use yazi_shim::cell::SyncCell;
use yazi_widgets::clear::ClearInventory;

use crate::{ADAPTOR, drivers::{Driver, Drivers}};

pub struct Adapter {
	driver:        Driver,
	shown:         SyncCell<Option<Rect>>,
	pub collision: SyncCell<bool>,
}

impl Deref for Adapter {
	type Target = Driver;

	fn deref(&self) -> &Self::Target { &self.driver }
}

impl Debug for Adapter {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { self.driver.fmt(f) }
}

impl Adapter {
	pub(super) fn new() -> Self {
		Self {
			driver:    Drivers::matches(&EMULATOR),
			shown:     SyncCell::new(None),
			collision: SyncCell::new(false),
		}
	}

	pub fn image_hide(&self) -> Result<()> {
		if let Some(area) = self.shown.replace(None) { self.driver.image_erase(area) } else { Ok(()) }
	}

	pub(super) fn shown_store(&self, area: Rect) { self.shown.set(Some(area)); }
}

inventory::submit! {
	ClearInventory {
		clear: |area| {
			let overlap = area.intersection(ADAPTOR.shown.get()?);
			if overlap.area() == 0 {
				return None;
			}

			ADAPTOR.driver.image_erase(overlap).ok();
			ADAPTOR.collision.set(true);
			Some(overlap)
		},
	}
}
