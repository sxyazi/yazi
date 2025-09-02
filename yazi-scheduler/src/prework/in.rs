use std::sync::Arc;

use yazi_config::plugin::{Fetcher, Preloader};
use yazi_shared::{Id, Throttle, url::UrlBuf};

#[derive(Debug)]
pub(crate) struct PreworkInFetch {
	pub(crate) id:      Id,
	pub(crate) plugin:  &'static Fetcher,
	pub(crate) targets: Vec<yazi_fs::File>,
}

#[derive(Clone, Debug)]
pub(crate) struct PreworkInLoad {
	pub(crate) id:     Id,
	pub(crate) plugin: &'static Preloader,
	pub(crate) target: yazi_fs::File,
}

#[derive(Debug)]
pub(crate) struct PreworkInSize {
	pub(crate) id:       Id,
	pub(crate) target:   UrlBuf,
	pub(crate) throttle: Arc<Throttle<(UrlBuf, u64)>>,
}
