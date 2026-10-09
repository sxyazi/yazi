use yazi_fs::file::File;
use yazi_macro::impl_data_any;
use yazi_shared::{event::{ActionCow, FromAction}, url::UrlBuf};

// --- OpenOpt
#[derive(Clone, Debug)]
pub struct OpenOpt {
	pub cwd:         Option<UrlBuf>,
	pub targets:     Vec<UrlBuf>,
	pub interactive: bool,
	pub hovered:     bool,
}

impl_data_any!(OpenOpt);

impl<C> FromAction<C> for OpenOpt {
	fn from_action(mut a: ActionCow, _: &C) -> anyhow::Result<Self> {
		Ok(Self {
			cwd:         a.take("cwd").ok(),
			targets:     a.take_seq(),
			interactive: a.bool("interactive"),
			hovered:     a.bool("hovered"),
		})
	}
}

// --- OpenDoOpt
#[derive(Clone, Debug, Default)]
pub struct OpenDoOpt {
	pub cwd:         UrlBuf,
	pub targets:     Vec<File>,
	pub interactive: bool,
}

impl_data_any!(OpenDoOpt);
