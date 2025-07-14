use anyhow::bail;
use yazi_shared::{Layer, event::CmdCow};

pub struct ToggleOpt {
	pub layer: Layer,
}

impl TryFrom<CmdCow> for ToggleOpt {
	type Error = anyhow::Error;

	fn try_from(c: CmdCow) -> Result<Self, Self::Error> {
		let Some(layer) = c.first_str() else {
			bail!("Invalid 'layer' in Toggle");
		};

		Ok(Self { layer: layer.parse()? })
	}
}

impl From<Layer> for ToggleOpt {
	fn from(layer: Layer) -> Self { Self { layer } }
}
