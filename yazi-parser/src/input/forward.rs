use yazi_shared::event::CmdCow;

pub struct ForwardOpt {
	pub far:         bool,
	pub end_of_word: bool,
}

impl From<CmdCow> for ForwardOpt {
	fn from(c: CmdCow) -> Self { Self { far: c.bool("far"), end_of_word: c.bool("end-of-word") } }
}
