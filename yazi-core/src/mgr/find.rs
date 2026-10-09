use anyhow::bail;
use yazi_fs::FilterCase;
use yazi_macro::impl_data_any;
use yazi_shared::event::{ActionCow, FromAction};
use yazi_shim::SStr;

#[derive(Clone, Debug)]
pub struct FindDoOpt {
	pub query: SStr,
	pub prev:  bool,
	pub case:  FilterCase,
}

impl_data_any!(FindDoOpt);

impl<C> FromAction<C> for FindDoOpt {
	fn from_action(mut a: ActionCow, _: &C) -> anyhow::Result<Self> {
		let Ok(query) = a.take_first() else {
			bail!("Invalid 'query' in FindDoOpt");
		};

		Ok(Self { query, prev: a.bool("previous"), case: FilterCase::from(&*a) })
	}
}
