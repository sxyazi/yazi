use yazi_shared::url::UrlBuf;

#[derive(Clone, Debug)]
pub struct DisplaceOpt {
	pub to:   Result<UrlBuf, yazi_fs::error::Error>,
	pub from: UrlBuf,
}
