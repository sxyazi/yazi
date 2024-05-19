use std::sync::Arc;

use yazi_config::plugin::PreloaderProps;
use yazi_shared::{fs::Url, Throttle};

#[derive(Debug)]
pub enum PreloadOp {
	Rule(PreloadOpRule),
	Size(PreloadOpSize),
}

impl PreloadOp {
	pub fn id(&self) -> usize {
		match self {
			Self::Rule(op) => op.id,
			Self::Size(op) => op.id,
		}
	}
}

#[derive(Clone, Debug)]
pub struct PreloadOpRule {
	pub id:      usize,
	pub plugin:  PreloaderProps,
	pub targets: Vec<yazi_shared::fs::File>,
}

#[derive(Debug)]
pub struct PreloadOpSize {
	pub id:       usize,
	pub target:   Url,
	pub throttle: Arc<Throttle<(Url, u64)>>,
}
