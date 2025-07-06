use yazi_fs::expand_path;
use yazi_shared::{event::CmdCow, url::Url};

pub struct CdOpt {
	pub target:      Url,
	pub interactive: bool,
	pub source:      CdSource,
}

impl From<CmdCow> for CdOpt {
	fn from(mut c: CmdCow) -> Self {
		let mut target = c.take_first_url().unwrap_or_default();
		if target.is_regular() && !c.bool("raw") {
			target = Url::from(expand_path(target));
		}
		Self { target, interactive: c.bool("interactive"), source: CdSource::Cd }
	}
}

impl From<(Url, CdSource)> for CdOpt {
	fn from((target, source): (Url, CdSource)) -> Self { Self { target, interactive: false, source } }
}

// --- Source
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CdSource {
	Tab,
	Cd,
	Reveal,
	Enter,
	Leave,
	Forward,
	Back,
}

impl CdSource {
	#[inline]
	pub fn big_jump(self) -> bool { self == Self::Cd || self == Self::Reveal }
}
