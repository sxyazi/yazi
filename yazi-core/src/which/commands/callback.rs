use tokio::sync::mpsc;
use tracing::error;
use yazi_shared::event::{CmdCow, Data};

use crate::which::Which;

pub struct Opt {
	tx:  mpsc::Sender<usize>,
	idx: usize,
}

impl TryFrom<CmdCow> for Opt {
	type Error = ();

	fn try_from(mut c: CmdCow) -> Result<Self, Self::Error> {
		Ok(Self {
			tx:  c.take_any("tx").ok_or(())?,
			idx: c.first().and_then(Data::as_usize).ok_or(())?,
		})
	}
}

impl Which {
	pub fn callback(&mut self, opt: impl TryInto<Opt>) {
		let Ok(opt) = opt.try_into() else {
			return;
		};

		if opt.tx.try_send(opt.idx).is_err() {
			error!("which callback: send error");
		}
	}
}
