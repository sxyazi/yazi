use std::borrow::Cow;

use yazi_shared::event::CmdCow;

pub struct SwipeOpt {
	pub step: Cow<'static, str>,
}

impl From<CmdCow> for SwipeOpt {
	fn from(mut c: CmdCow) -> Self { Self { step: c.take_first_str().unwrap_or_default() } }
}
