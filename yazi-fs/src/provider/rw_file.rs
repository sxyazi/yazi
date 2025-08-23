use crate::provider::BufRead;

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
}
