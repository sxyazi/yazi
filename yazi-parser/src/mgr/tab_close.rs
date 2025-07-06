use yazi_shared::event::{CmdCow, Data};

pub struct TabCloseOpt {
	pub idx: usize,
}

impl From<CmdCow> for TabCloseOpt {
	fn from(c: CmdCow) -> Self { Self { idx: c.first().and_then(Data::as_usize).unwrap_or(0) } }
}

impl From<usize> for TabCloseOpt {
	fn from(idx: usize) -> Self { Self { idx } }
}
