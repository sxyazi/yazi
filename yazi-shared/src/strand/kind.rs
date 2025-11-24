use crate::{path::PathKind, scheme::SchemeKind};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum StrandKind {
	Utf8  = 0,
	Os    = 1,
	Bytes = 2,
}

impl From<PathKind> for StrandKind {
	fn from(value: PathKind) -> Self {
		match value {
			PathKind::Os => StrandKind::Os,
			PathKind::Unix => StrandKind::Bytes,
		}
	}
}

impl From<SchemeKind> for StrandKind {
	fn from(value: SchemeKind) -> Self {
		match value {
			SchemeKind::Regular => Self::Os,
			SchemeKind::Search => Self::Os,
			SchemeKind::Archive => Self::Os,
			SchemeKind::Sftp => Self::Os, // FIXME
		}
	}
}
