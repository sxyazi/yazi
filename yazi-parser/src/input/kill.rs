use yazi_shared::{SStr, event::CmdCow};

pub struct KillOpt {
	pub kind: SStr,
}

impl From<CmdCow> for KillOpt {
	fn from(mut c: CmdCow) -> Self { Self { kind: c.take_first_str().unwrap_or_default() } }
}
