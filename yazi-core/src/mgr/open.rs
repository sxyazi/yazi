use yazi_fs::File;
use yazi_shared::{event::ActionCow, url::UrlBuf};

#[derive(Clone, Debug)]
pub struct OpenOpt {
	pub cwd:         Option<UrlBuf>,
	pub targets:     Vec<UrlBuf>,
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
	pub cwd:         UrlBuf,
	pub targets:     Vec<File>,
	pub interactive: bool,
}
