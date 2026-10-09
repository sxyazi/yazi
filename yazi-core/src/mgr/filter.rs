use yazi_fs::FilterCase;
use yazi_macro::impl_data_any;
use yazi_shared::event::{ActionCow, FromAction};
use yazi_shim::SStr;

#[derive(Clone, Debug, Default)]
pub struct FilterOpt {
	pub query: SStr,
	pub case:  FilterCase,
	pub done:  bool,
}

impl_data_any!(FilterOpt);

impl<C> FromAction<C> for FilterOpt {
	fn from_action(mut a: ActionCow, _: &C) -> anyhow::Result<Self> {
		Ok(Self {
			query: a.take_first().unwrap_or_default(),
			case:  FilterCase::from(&*a),
			done:  a.bool("done"),
		})
	}
}
