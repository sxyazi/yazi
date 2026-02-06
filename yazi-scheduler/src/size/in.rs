use std::sync::Arc;

use yazi_shared::{Id, Throttle, url::UrlBuf};

#[derive(Debug)]
pub(crate) struct SizeIn {
	pub(crate) id:       Id,
	pub(crate) target:   UrlBuf,
	pub(crate) throttle: Arc<Throttle<(UrlBuf, u64)>>,
}

impl SizeIn {
	pub(crate) fn id(&self) -> Id { self.id }
}
