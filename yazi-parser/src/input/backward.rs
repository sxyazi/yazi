use yazi_shared::event::CmdCow;

pub struct BackwardOpt {
	pub far: bool,
}

impl From<CmdCow> for BackwardOpt {
	fn from(c: CmdCow) -> Self { Self { far: c.bool("far") } }
}
