use yazi_macro::impl_data_any;
use yazi_shared::url::UrlBuf;

#[derive(Clone, Debug)]
pub struct DisplaceOpt {
	pub to:   Result<UrlBuf, yazi_fs::error::Error>,
	pub from: UrlBuf,
}

impl_data_any!(DisplaceOpt);
