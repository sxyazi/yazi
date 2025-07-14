use yazi_shared::event::CmdCow;

pub struct HiddenOpt {
	pub state: Option<bool>,
}

impl From<CmdCow> for HiddenOpt {
	fn from(c: CmdCow) -> Self {
		let state = match c.first_str() {
			Some("show") => Some(true),
			Some("hide") => Some(false),
			_ => None,
		};

		Self { state }
	}
}
