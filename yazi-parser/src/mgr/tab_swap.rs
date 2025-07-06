use yazi_shared::event::{CmdCow, Data};

pub struct TabSwapOpt {
	pub step: isize,
}

impl From<CmdCow> for TabSwapOpt {
	fn from(c: CmdCow) -> Self { Self { step: c.first().and_then(Data::as_isize).unwrap_or(0) } }
}
