use yazi_shared::{event::{CmdCow, Data}, url::Url};

#[derive(Default)]
pub struct UpdatePagedOpt {
	pub page:    Option<usize>,
	pub only_if: Option<Url>,
}

impl From<CmdCow> for UpdatePagedOpt {
	fn from(mut c: CmdCow) -> Self {
		Self { page: c.first().and_then(Data::as_usize), only_if: c.take_url("only-if") }
	}
}

impl From<()> for UpdatePagedOpt {
	fn from(_: ()) -> Self { Self::default() }
}
