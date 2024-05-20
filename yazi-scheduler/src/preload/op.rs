use std::sync::Arc;

use yazi_config::plugin::{PrefetcherProps, PreloaderProps};
use yazi_shared::{fs::Url, Throttle};

#[derive(Debug)]
pub enum PreworkOp {
	Fetch(PreloadOpFetch),
	Load(PreloadOpLoad),
	Size(PreloadOpSize),
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
pub struct PreloadOpFetch {
	pub id:      usize,
	pub plugin:  PrefetcherProps,
	pub targets: Vec<yazi_shared::fs::File>,
}

#[derive(Clone, Debug)]
pub struct PreloadOpLoad {
	pub id:     usize,
	pub plugin: PreloaderProps,
	pub target: yazi_shared::fs::File,
}

#[derive(Debug)]
pub struct PreloadOpSize {
	pub id:       usize,
	pub target:   Url,
	pub throttle: Arc<Throttle<(Url, u64)>>,
}
