use yazi_shared::event::CmdCow;

pub struct CreateOpt {
	pub dir:   bool,
	pub force: bool,
}

impl From<CmdCow> for CreateOpt {
	fn from(c: CmdCow) -> Self { Self { dir: c.bool("dir"), force: c.bool("force") } }
}
