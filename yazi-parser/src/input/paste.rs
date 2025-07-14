use yazi_shared::event::CmdCow;

pub struct PasteOpt {
	pub before: bool,
}

impl From<CmdCow> for PasteOpt {
	fn from(c: CmdCow) -> Self { Self { before: c.bool("before") } }
}
