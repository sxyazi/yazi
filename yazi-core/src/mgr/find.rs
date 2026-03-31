use anyhow::bail;
use yazi_fs::FilterCase;
use yazi_shared::{SStr, event::ActionCow};

#[derive(Clone, Debug)]
pub struct FindDoOpt {
	pub query: SStr,
	pub prev:  bool,
	pub case:  FilterCase,
}

impl TryFrom<ActionCow> for FindDoOpt {
	type Error = anyhow::Error;

	fn try_from(mut a: ActionCow) -> Result<Self, Self::Error> {
		let Ok(query) = a.take_first() else {
			bail!("Invalid 'query' in FindDoOpt");
		};

		Ok(Self { query, prev: a.bool("previous"), case: FilterCase::from(&*a) })
	}
}
