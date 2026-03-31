use yazi_shared::{event::ActionCow, url::UrlCow};

#[derive(Clone, Debug)]
pub struct OpenOpt {
	pub cwd:         Option<UrlCow<'static>>,
	pub targets:     Vec<UrlCow<'static>>,
	pub interactive: bool,
	pub hovered:     bool,
}

impl TryFrom<ActionCow> for OpenOpt {
	type Error = anyhow::Error;

	fn try_from(mut a: ActionCow) -> Result<Self, Self::Error> {
		Ok(Self {
			cwd:         a.take("cwd").ok(),
			targets:     a.take_seq(),
			interactive: a.bool("interactive"),
			hovered:     a.bool("hovered"),
		})
	}
}

// OpenDoOpt
#[derive(Clone, Debug, Default)]
pub struct OpenDoOpt {
	pub cwd:         UrlCow<'static>,
	pub targets:     Vec<UrlCow<'static>>,
	pub interactive: bool,
}
