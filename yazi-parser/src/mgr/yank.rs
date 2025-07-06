use yazi_shared::event::CmdCow;

pub struct YankOpt {
	pub cut: bool,
}

impl From<CmdCow> for YankOpt {
	fn from(c: CmdCow) -> Self { Self { cut: c.bool("cut") } }
}
