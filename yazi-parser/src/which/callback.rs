use anyhow::bail;
use tokio::sync::mpsc;
use yazi_shared::event::{CmdCow, Data};

pub struct CallbackOpt {
	pub tx:  mpsc::Sender<usize>,
	pub idx: usize,
}

impl TryFrom<CmdCow> for CallbackOpt {
	type Error = anyhow::Error;

	fn try_from(mut c: CmdCow) -> Result<Self, Self::Error> {
		let Some(tx) = c.take_any("tx") else {
			bail!("Invalid 'tx' argument in CallbackOpt");
		};

		let Some(idx) = c.first().and_then(Data::as_usize) else {
			bail!("Invalid 'idx' argument in CallbackOpt");
		};

		Ok(Self { tx, idx })
	}
}
