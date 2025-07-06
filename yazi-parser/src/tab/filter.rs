use std::borrow::Cow;

use yazi_fs::FilterCase;
use yazi_shared::event::CmdCow;

#[derive(Default)]
pub struct FilterOpt {
	pub query: Cow<'static, str>,
	pub case:  FilterCase,
	pub done:  bool,
}

impl From<CmdCow> for FilterOpt {
	fn from(mut c: CmdCow) -> Self {
		Self {
			query: c.take_first_str().unwrap_or_default(),
			case:  FilterCase::from(&*c),
			done:  c.bool("done"),
		}
	}
}
