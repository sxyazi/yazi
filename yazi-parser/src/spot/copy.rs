use yazi_shared::{SStr, event::CmdCow};

pub struct CopyOpt {
	pub r#type: SStr,
}

impl From<CmdCow> for CopyOpt {
	fn from(mut c: CmdCow) -> Self { Self { r#type: c.take_first_str().unwrap_or_default() } }
}
