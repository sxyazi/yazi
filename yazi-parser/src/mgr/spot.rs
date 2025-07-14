use yazi_shared::event::{CmdCow, Data};

#[derive(Default)]
pub struct SpotOpt {
	pub skip: Option<usize>,
}

impl From<CmdCow> for SpotOpt {
	fn from(c: CmdCow) -> Self { Self { skip: c.get("skip").and_then(Data::as_usize) } }
}

impl From<usize> for SpotOpt {
	fn from(skip: usize) -> Self { Self { skip: Some(skip) } }
}
