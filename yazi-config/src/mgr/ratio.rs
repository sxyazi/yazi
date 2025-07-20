use anyhow::bail;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(try_from = "[u16; 3]")]
pub struct MgrRatio {
	pub parent:  u16,
	pub current: u16,
	pub preview: u16,
	pub all:     u16,
}

impl TryFrom<[u16; 3]> for MgrRatio {
	type Error = anyhow::Error;

	fn try_from(ratio: [u16; 3]) -> Result<Self, Self::Error> {
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
