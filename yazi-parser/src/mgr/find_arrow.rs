use yazi_shared::event::CmdCow;

pub struct FindArrowOpt {
	pub prev: bool,
}

impl From<CmdCow> for FindArrowOpt {
	fn from(c: CmdCow) -> Self { Self { prev: c.bool("previous") } }
}
