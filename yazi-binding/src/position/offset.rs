use anyhow::bail;
use serde::Deserialize;

#[derive(Clone, Copy, Debug, Default, Deserialize)]
#[serde(try_from = "[i16; 4]")]
pub struct Offset {
	pub x:      i16,
	pub y:      i16,
	pub width:  u16,
	pub height: u16,
}

impl From<ratatui_core::layout::Rect> for Offset {
	fn from(value: ratatui_core::layout::Rect) -> Self {
		Self {
			x:      value.x as i16,
			y:      value.y as i16,
			width:  value.width,
			height: value.height,
		}
	}
}

impl TryFrom<[i16; 4]> for Offset {
	type Error = anyhow::Error;

	fn try_from(values: [i16; 4]) -> Result<Self, Self::Error> {
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
