use std::ops::Deref;

use parking_lot::RwLock;
use yazi_shared::RoCell;

use super::Partition;
use crate::cha::Cha;

pub(super) type Locked = RwLock<Partitions>;

pub static PARTITIONS: RoCell<Locked> = RoCell::new();

#[derive(Default)]
pub struct Partitions {
	pub(super) inner:       Vec<Partition>,
	#[cfg(target_os = "linux")]
	pub(super) linux_cache: hashbrown::HashSet<String>,
	#[cfg(target_os = "macos")]
	pub(super) need_update: bool,
}

impl Deref for Partitions {
	type Target = Vec<Partition>;

	fn deref(&self) -> &Self::Target { &self.inner }
}

impl Partitions {
	#[cfg(unix)]
	pub fn by_dev(&self, dev: u64) -> Option<&Partition> {
		self.inner.iter().find(|p| p.rdev == Some(dev))
	}

	pub fn heuristic(&self, _cha: Cha) -> bool {
		#[cfg(any(target_os = "linux", target_os = "macos"))]
		{
			self.by_dev(_cha.dev).is_none_or(|p| p.heuristic())
		}
		#[cfg(not(any(target_os = "linux", target_os = "macos")))]
		{
			true
		}
	}
}
