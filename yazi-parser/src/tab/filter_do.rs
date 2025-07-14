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
		let Some(query) = c.take_first_str() else {
			bail!("'query' is required for FindOpt");
		};

		Ok(Self { query, prev: c.bool("previous"), case: FilterCase::from(&*c) })
	}
}
