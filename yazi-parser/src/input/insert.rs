use yazi_shared::event::CmdCow;

pub struct InsertOpt {
	pub append: bool,
}

impl From<CmdCow> for InsertOpt {
	fn from(c: CmdCow) -> Self { Self { append: c.bool("append") } }
}

impl From<bool> for InsertOpt {
	fn from(append: bool) -> Self { Self { append } }
}
