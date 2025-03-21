use std::borrow::Cow;

use yazi_config::opener::OpenerRule;
use yazi_shared::{event::CmdCow, url::Url};

// --- Open
#[derive(Default)]
pub struct OpenDoOpt {
	pub cwd:         Url,
	pub hovered:     Url,
	pub targets:     Vec<(Url, &'static str)>,
	pub interactive: bool,
}

impl From<CmdCow> for OpenDoOpt {
	fn from(mut c: CmdCow) -> Self { c.take_any("option").unwrap_or_default() }
}

// --- Open with
pub struct OpenWithOpt {
	pub opener:  Cow<'static, OpenerRule>,
	pub cwd:     Url,
	pub targets: Vec<Url>,
}

impl TryFrom<CmdCow> for OpenWithOpt {
	type Error = ();

	fn try_from(mut c: CmdCow) -> Result<Self, Self::Error> { c.take_any("option").ok_or(()) }
}
