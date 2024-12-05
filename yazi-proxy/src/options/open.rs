use std::borrow::Cow;

use yazi_config::open::Opener;
use yazi_shared::{event::CmdCow, url::Url};

// --- Open
#[derive(Default)]
pub struct OpenDoOpt {
	pub hovered:     Url,
	pub targets:     Vec<(Url, Cow<'static, str>)>,
	pub interactive: bool,
}

impl From<CmdCow> for OpenDoOpt {
	fn from(mut c: CmdCow) -> Self { c.take_any("option").unwrap_or_default() }
}

// --- Open with
pub struct OpenWithOpt {
	pub targets: Vec<Url>,
	pub opener:  Cow<'static, Opener>,
}

impl TryFrom<CmdCow> for OpenWithOpt {
	type Error = ();

	fn try_from(mut c: CmdCow) -> Result<Self, Self::Error> { c.take_any("option").ok_or(()) }
}
