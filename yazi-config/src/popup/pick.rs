use serde::Deserialize;
use yazi_binding::position::{Offset, Origin, Position};
use yazi_codegen::{DeserializeOver, DeserializeOver2};

use crate::popup::PickCfg;

#[derive(Deserialize, DeserializeOver, DeserializeOver2)]
pub struct Pick {
	// open
	pub open_title:  String,
	pub open_origin: Origin,
	pub open_offset: Offset,
}

impl Pick {
	pub const BORDER: u16 = 2;

	fn max_height(&self, len: usize) -> u16 {
		self.open_offset.height.min(Self::BORDER.saturating_add(len as u16))
	}

	pub fn open(&self, items: Vec<String>) -> PickCfg {
		let max_height = self.max_height(items.len());
		PickCfg {
			title: self.open_title.clone(),
			items,
			position: Position::new(self.open_origin, Offset { height: max_height, ..self.open_offset }),
		}
	}
}
