use yazi_shared::event::CmdCow;

use crate::mgr::QuitOpt;

#[derive(Default)]
pub struct CloseOpt(pub QuitOpt);

impl From<CmdCow> for CloseOpt {
	fn from(c: CmdCow) -> Self { Self(c.into()) }
}
