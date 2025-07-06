use tokio::sync::mpsc;
use yazi_shared::event::{CmdCow, Data};

pub struct CallbackOpt {
	pub tx:  mpsc::Sender<usize>,
	pub idx: usize,
}

impl TryFrom<CmdCow> for CallbackOpt {
	type Error = ();

	fn try_from(mut c: CmdCow) -> Result<Self, Self::Error> {
		Ok(Self {
			tx:  c.take_any("tx").ok_or(())?,
			idx: c.first().and_then(Data::as_usize).ok_or(())?,
		})
	}
}
