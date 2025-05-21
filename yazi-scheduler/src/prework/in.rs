use std::sync::Arc;

use yazi_config::plugin::{Fetcher, Preloader};
use yazi_shared::{Throttle, url::Url};

#[derive(Debug)]
pub enum PreworkIn {
	Fetch(PreworkInFetch),
	Load(PreworkInLoad),
	Size(PreworkInSize),
}

impl PreworkIn {
	pub fn id(&self) -> usize {
		match self {
			Self::Fetch(r#in) => r#in.id,
			Self::Load(r#in) => r#in.id,
			Self::Size(r#in) => r#in.id,
		}
	}
}

#[derive(Debug)]
pub struct PreworkInFetch {
	pub id:      usize,
	pub plugin:  &'static Fetcher,
	pub targets: Vec<yazi_fs::File>,
}

#[derive(Clone, Debug)]
pub struct PreworkInLoad {
	pub id:     usize,
	pub plugin: &'static Preloader,
	pub target: yazi_fs::File,
}

#[derive(Debug)]
pub struct PreworkInSize {
	pub id:       usize,
	pub target:   Url,
	pub throttle: Arc<Throttle<(Url, u64)>>,
}
