use std::borrow::Cow;

use yazi_shared::event::CmdCow;

pub struct CopyOpt {
	pub r#type: Cow<'static, str>,
}

impl From<CmdCow> for CopyOpt {
	fn from(mut c: CmdCow) -> Self { Self { r#type: c.take_first_str().unwrap_or_default() } }
}
