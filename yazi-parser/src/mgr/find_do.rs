use anyhow::bail;
use yazi_fs::FilterCase;
use yazi_shared::{SStr, event::CmdCow};

pub struct FindDoOpt {
	pub query: SStr,
	pub prev:  bool,
	pub case:  FilterCase,
}

impl TryFrom<CmdCow> for FindDoOpt {
	type Error = anyhow::Error;

	fn try_from(mut c: CmdCow) -> Result<Self, Self::Error> {
		if let Some(opt) = c.take_any2("opt") {
			return opt;
		}

		let Some(query) = c.take_first_str() else {
			bail!("'query' is required for FindDoOpt");
		};

		Ok(Self { query, prev: c.bool("previous"), case: FilterCase::from(&*c) })
	}
}
