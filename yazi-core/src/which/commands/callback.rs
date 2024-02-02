use tokio::sync::mpsc;
use yazi_shared::event::Cmd;

use crate::which::Which;

pub struct Opt {
	tx:  mpsc::Sender<usize>,
	idx: usize,
}

impl TryFrom<Cmd> for Opt {
	type Error = ();

	fn try_from(mut c: Cmd) -> Result<Self, Self::Error> {
		Ok(Self {
			tx:  c.take_data().ok_or(())?,
			idx: c.take_first().and_then(|s| s.parse().ok()).ok_or(())?,
		})
	}
}

impl Which {
	pub fn callback(&mut self, opt: impl TryInto<Opt>) {
		if let Ok(opt) = opt.try_into() {
			opt.tx.try_send(opt.idx).ok();
		}
	}
}
