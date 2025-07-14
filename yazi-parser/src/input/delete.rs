use yazi_shared::event::CmdCow;

pub struct DeleteOpt {
	pub cut:    bool,
	pub insert: bool,
}

impl From<CmdCow> for DeleteOpt {
	fn from(c: CmdCow) -> Self { Self { cut: c.bool("cut"), insert: c.bool("insert") } }
}
