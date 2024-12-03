use std::sync::Arc;

use yazi_config::plugin::{Fetcher, Preloader};
use yazi_shared::{Throttle, url::Url};

#[derive(Debug)]
pub enum PreworkOp {
	Fetch(PreworkOpFetch),
	Load(PreworkOpLoad),
	Size(PreworkOpSize),
}

impl PreworkOp {
	pub fn id(&self) -> usize {
		match self {
			Self::Fetch(op) => op.id,
			Self::Load(op) => op.id,
			Self::Size(op) => op.id,
		}
	}
}

#[derive(Debug)]
pub struct PreworkOpFetch {
	pub id:      usize,
	pub plugin:  &'static Fetcher,
	pub targets: Vec<yazi_fs::File>,
}

#[derive(Clone, Debug)]
pub struct PreworkOpLoad {
	pub id:     usize,
	pub plugin: &'static Preloader,
	pub target: yazi_fs::File,
}

#[derive(Debug)]
pub struct PreworkOpSize {
	pub id:       usize,
	pub target:   Url,
	pub throttle: Arc<Throttle<(Url, u64)>>,
}
