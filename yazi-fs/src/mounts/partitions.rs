use std::{ops::{Deref, DerefMut}, path::Path};

use parking_lot::RwLock;

use super::Partition;
use crate::stat::Stat;

pub(super) type Locked = RwLock<Partitions>;

pub static PARTITIONS: Locked = RwLock::new(Partitions { inner: Vec::new(), loaded: false });

#[derive(Debug, Default)]
pub struct Partitions {
	pub(super) inner: Vec<Partition>,
	loaded:           bool,
}

impl Deref for Partitions {
	type Target = Vec<Partition>;

	fn deref(&self) -> &Self::Target { &self.inner }
}

impl DerefMut for Partitions {
	fn deref_mut(&mut self) -> &mut Self::Target { &mut self.inner }
}

impl From<Vec<Partition>> for Partitions {
	fn from(inner: Vec<Partition>) -> Self { Self { inner, loaded: true } }
}

impl FromIterator<Partition> for Partitions {
	fn from_iter<I>(iter: I) -> Self
	where
		I: IntoIterator<Item = Partition>,
	{
		Vec::from_iter(iter).into()
	}
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
			let new = vec![].into();

			*PARTITIONS.write() = new;
			Ok(())
		})
		.await?
	}

	#[cfg(unix)]
	fn by_dev(&self, dev: u64) -> Option<&Partition> { self.iter().find(|p| p.rdev == Some(dev)) }

	pub fn mount_point(&self, path: impl AsRef<Path>) -> Option<&Path> {
		let path = path.as_ref();

		self
			.iter()
			.filter_map(|p| p.dist.as_deref())
			.filter(|p| path.starts_with(p))
			.max_by_key(|p| p.as_os_str().len())
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
