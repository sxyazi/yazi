use yazi_shared::event::CmdCow;

#[derive(Clone, Copy)]
pub struct OpenOpt {
	pub interactive: bool,
	pub hovered:     bool,
}

impl From<CmdCow> for OpenOpt {
	fn from(c: CmdCow) -> Self {
		Self { interactive: c.bool("interactive"), hovered: c.bool("hovered") }
	}
}
