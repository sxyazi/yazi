use std::ops::Deref;

use parking_lot::RwLock;

use super::Partition;
use crate::stat::Stat;

pub(super) type Locked = RwLock<Partitions>;

pub static PARTITIONS: Locked = RwLock::new(Partitions { inner: Vec::new(), loaded: false });

#[derive(Default)]
pub struct Partitions {
	pub(super) inner: Vec<Partition>,
	loaded:           bool,
}

impl Deref for Partitions {
	type Target = Vec<Partition>;

	fn deref(&self) -> &Self::Target { &self.inner }
}

impl Partitions {
	pub async fn ensure() -> anyhow::Result<()> {
		if PARTITIONS.read().loaded {
			return Ok(());
		}

		tokio::task::spawn_blocking(|| {
			#[cfg(target_os = "macos")]
			let new = Self::all_partitions(Self::all_names()?)?;
			#[cfg(any(target_os = "linux", windows))]
			let new = Self::all()?;
			#[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
			let new = Vec::new();

			PARTITIONS.write().set(new);
			Ok(())
		})
		.await?
	}

	pub(super) fn set(&mut self, partitions: Vec<Partition>) {
		self.inner = partitions;
		self.loaded = true;
	}

	#[cfg(unix)]
	fn by_dev(&self, dev: u64) -> Option<&Partition> {
		self.inner.iter().find(|p| p.rdev == Some(dev))
	}

	pub fn timeless(&self, _stat: Stat) -> bool {
		#[cfg(any(target_os = "linux", target_os = "macos"))]
		{
			self.by_dev(_stat.dev).is_some_and(|p| p.timeless())
		}
		#[cfg(not(any(target_os = "linux", target_os = "macos")))]
		{
			// For now, assume other targets update directory mtime correctly
			false
		}
	}

	pub fn soundless(&self, _stat: Stat) -> bool {
		#[cfg(any(target_os = "linux", target_os = "macos"))]
		{
			self.by_dev(_stat.dev).is_some_and(|p| p.soundless())
		}
		#[cfg(not(any(target_os = "linux", target_os = "macos")))]
		{
			// For now, assume other targets emit change notifications correctly
			false
		}
	}
}
