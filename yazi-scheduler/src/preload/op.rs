use std::sync::Arc;

use yazi_shared::{fs::Url, Throttle};

#[derive(Debug)]
pub enum PreloadOp {
	Rule(PreloadOpRule),
	Size(PreloadOpSize),
}

#[derive(Clone, Debug)]
pub struct PreloadOpRule {
	pub id:         usize,
	pub rule_id:    u8,
	pub rule_multi: bool,
	pub plugin:     String,
	pub targets:    Vec<yazi_shared::fs::File>,
}

#[derive(Debug)]
pub struct PreloadOpSize {
	pub id:       usize,
	pub target:   Url,
	pub throttle: Arc<Throttle<(Url, u64)>>,
}
