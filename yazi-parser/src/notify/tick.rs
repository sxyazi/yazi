use std::time::Duration;

use yazi_shared::event::{Cmd, CmdCow, Data};

pub struct TickOpt {
	pub interval: Duration,
}

impl TryFrom<CmdCow> for TickOpt {
	type Error = ();

	fn try_from(c: CmdCow) -> Result<Self, Self::Error> {
		let interval = c.first().and_then(Data::as_f64).ok_or(())?;
		if interval < 0.0 {
			return Err(());
		}

		Ok(Self { interval: Duration::from_secs_f64(interval) })
	}
}

impl TryFrom<Cmd> for TickOpt {
	type Error = ();

	fn try_from(c: Cmd) -> Result<Self, Self::Error> { Self::try_from(CmdCow::from(c)) }
}
