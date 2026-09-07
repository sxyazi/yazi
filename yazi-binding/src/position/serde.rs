use serde::{Deserialize, Deserializer};

use super::{Offset, Origin, Position};

impl<'de> Deserialize<'de> for Position {
	fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
		#[derive(Deserialize)]
		struct Flat {
			at: Origin,
			#[serde(default)]
			x:  i16,
			#[serde(default)]
			y:  i16,
			#[serde(default)]
			w:  u16,
			#[serde(default)]
			h:  u16,
		}

		let Flat { at, x, y, w, h } = Flat::deserialize(de)?;
		Ok(Position::new(at, Offset { x, y, width: w, height: h }))
	}
}
