use std::io;

use super::Partitions;

impl Partitions {
	pub(crate) fn mounts() -> io::Result<Self> {
		Err(io::Error::new(
			io::ErrorKind::Unsupported,
			"mount points cannot be determined on this platform",
		))
	}
}
