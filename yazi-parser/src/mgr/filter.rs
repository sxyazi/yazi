use yazi_fs::FilterCase;
use yazi_shared::{SStr, event::CmdCow};

#[derive(Default)]
pub struct FilterOpt {
	pub query: SStr,
	pub case:  FilterCase,
	pub done:  bool,
}

impl TryFrom<CmdCow> for FilterOpt {
	type Error = anyhow::Error;

	fn try_from(mut c: CmdCow) -> Result<Self, Self::Error> {
		if let Some(opt) = c.take_any2("opt") {
			return opt;
		}

		Ok(Self {
			query: c.take_first_str().unwrap_or_default(),
			case:  FilterCase::from(&*c),
			done:  c.bool("done"),
		})
	}
}
