use crate::provider::{BufRead, BufReadSync};

pub enum RwFile {
	Local(super::local::RwFile),
}

impl RwFile {
	#[inline]
	pub fn reader(self) -> Box<dyn BufRead> {
		match self {
			RwFile::Local(local) => Box::new(local.reader()),
		}
	}

	#[inline]
	pub async fn reader_sync(self) -> Box<dyn BufReadSync> {
		match self {
			RwFile::Local(local) => Box::new(local.reader_sync().await),
		}
	}
}
