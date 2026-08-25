use yazi_macro::{emit, relay};
use yazi_shared::url::UrlBuf;

pub struct MgrProxy;

impl MgrProxy {
	pub(crate) fn refresh() {
		emit!(Call(relay!(mgr:refresh)));
	}

	pub(crate) fn upload<I>(urls: I)
	where
		I: IntoIterator<Item = UrlBuf>,
	{
		emit!(Call(relay!(mgr:upload).with_seq(urls)));
	}

	pub fn watch() {
		emit!(Call(relay!(mgr:watch)));
	}
}
