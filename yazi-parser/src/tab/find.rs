use std::borrow::Cow;

use yazi_fs::FilterCase;
use yazi_shared::event::CmdCow;

pub struct FindOpt {
	pub query: Option<Cow<'static, str>>,
	pub prev:  bool,
	pub case:  FilterCase,
}

impl From<CmdCow> for FindOpt {
	fn from(mut c: CmdCow) -> Self {
		Self { query: c.take_first_str(), prev: c.bool("previous"), case: FilterCase::from(&*c) }
	}
}
