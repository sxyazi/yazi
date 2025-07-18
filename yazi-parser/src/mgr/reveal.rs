use yazi_fs::expand_path;
use yazi_shared::{event::CmdCow, url::Url};

use crate::mgr::CdSource;

pub struct RevealOpt {
	pub target:   Url,
	pub source:   CdSource,
	pub no_dummy: bool,
}

impl From<CmdCow> for RevealOpt {
	fn from(mut c: CmdCow) -> Self {
		let mut target = c.take_first_url().unwrap_or_default();
		if target.is_regular() && !c.bool("raw") {
			target = Url::from(expand_path(target));
		}

		Self { target, source: CdSource::Reveal, no_dummy: c.bool("no-dummy") }
	}
}

impl From<Url> for RevealOpt {
	fn from(target: Url) -> Self { Self { target, source: CdSource::Reveal, no_dummy: false } }
}

impl From<(Url, CdSource)> for RevealOpt {
	fn from((target, source): (Url, CdSource)) -> Self { Self { target, source, no_dummy: false } }
}
