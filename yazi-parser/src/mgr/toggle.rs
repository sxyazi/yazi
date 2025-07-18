use yazi_shared::{event::CmdCow, url::Url};

pub struct ToggleOpt {
	pub url:   Option<Url>,
	pub state: Option<bool>,
}

impl From<CmdCow> for ToggleOpt {
	fn from(mut c: CmdCow) -> Self {
		Self {
			url:   c.take_first_url(),
			state: match c.str("state") {
				Some("on") => Some(true),
				Some("off") => Some(false),
				_ => None,
			},
		}
	}
}
