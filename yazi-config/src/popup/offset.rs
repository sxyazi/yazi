use anyhow::bail;
use serde::Deserialize;

#[derive(Clone, Copy, Debug, Default, Deserialize)]
#[serde(try_from = "Vec<i16>")]
pub struct Offset {
	pub x:      i16,
	pub y:      i16,
	pub width:  u16,
	pub height: u16,
}

impl TryFrom<Vec<i16>> for Offset {
	type Error = anyhow::Error;

	fn try_from(values: Vec<i16>) -> Result<Self, Self::Error> {
		if values.len() != 4 {
			bail!("offset must have 4 values: {:?}", values);
		}
		if values[2] < 0 || values[3] < 0 {
			bail!("offset width and height must be positive: {:?}", values);
		}
		if values[3] < 3 {
			bail!("offset height must be at least 3: {:?}", values);
		}

		Ok(Self {
			x:      values[0],
			y:      values[1],
			width:  values[2] as u16,
			height: values[3] as u16,
		})
	}
}

impl Offset {
	#[inline]
	pub fn line() -> Self { Self { x: 0, y: 0, width: u16::MAX, height: 1 } }
}
