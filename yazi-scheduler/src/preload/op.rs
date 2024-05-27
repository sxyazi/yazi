use std::sync::Arc;

use yazi_config::plugin::{FetcherProps, PreloaderProps};
use yazi_shared::{fs::Url, Throttle};

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

#[derive(Clone, Debug)]
pub struct PreworkOpFetch {
	pub id:      usize,
	pub plugin:  FetcherProps,
	pub targets: Vec<yazi_shared::fs::File>,
}

#[derive(Clone, Debug)]
pub struct PreworkOpLoad {
	pub id:     usize,
	pub plugin: PreloaderProps,
	pub target: yazi_shared::fs::File,
}

#[derive(Debug)]
pub struct PreworkOpSize {
	pub id:       usize,
	pub target:   Url,
	pub throttle: Arc<Throttle<(Url, u64)>>,
}
