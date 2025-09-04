use crate::provider::BufRead;

pub enum RwFile {
	Local(super::local::RwFile),
}

impl RwFile {
	pub fn reader(self) -> Box<dyn BufRead> {
		match self {
			Self::Local(local) => Box::new(local.reader()),
		}
	}
}
