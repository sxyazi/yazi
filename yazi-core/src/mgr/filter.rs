use yazi_fs::FilterCase;
use yazi_macro::impl_data_any;
use yazi_shared::event::ActionCow;
use yazi_shim::SStr;

#[derive(Clone, Debug, Default)]
pub struct FilterOpt {
	pub query: SStr,
	pub case:  FilterCase,
	pub done:  bool,
}

impl_data_any!(FilterOpt);

impl TryFrom<ActionCow> for FilterOpt {
	type Error = anyhow::Error;

	fn try_from(mut a: ActionCow) -> Result<Self, Self::Error> {
		Ok(Self {
			query: a.take_first().unwrap_or_default(),
			case:  FilterCase::from(&*a),
			done:  a.bool("done"),
		})
	}
}
