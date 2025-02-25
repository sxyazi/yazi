use anyhow::bail;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(try_from = "Vec<u16>")]
pub struct MgrRatio {
	pub parent:  u16,
	pub current: u16,
	pub preview: u16,
	pub all:     u16,
}

impl TryFrom<Vec<u16>> for MgrRatio {
	type Error = anyhow::Error;

	fn try_from(ratio: Vec<u16>) -> Result<Self, Self::Error> {
		if ratio.len() != 3 {
			bail!("invalid layout ratio: {:?}", ratio);
		}
		if ratio.iter().all(|&r| r == 0) {
			bail!("at least one layout ratio must be non-zero: {:?}", ratio);
		}

		Ok(Self {
			parent:  ratio[0],
			current: ratio[1],
			preview: ratio[2],
			all:     ratio[0] + ratio[1] + ratio[2],
		})
	}
}
