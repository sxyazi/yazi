use std::time::Duration;

use anyhow::bail;
use yazi_shared::event::{CmdCow, Data};

pub struct TickOpt {
	pub interval: Duration,
}

impl TryFrom<CmdCow> for TickOpt {
	type Error = anyhow::Error;

	fn try_from(c: CmdCow) -> Result<Self, Self::Error> {
		let Some(interval) = c.first().and_then(Data::as_f64) else {
			bail!("Invalid 'interval' argument in TickOpt");
		};

		if interval < 0.0 {
			bail!("'interval' must be non-negative in TickOpt");
		}

		Ok(Self { interval: Duration::from_secs_f64(interval) })
	}
}
