use crate::{auth::AuthKind, path::PathKind};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum StrandKind {
	Utf8  = 0,
	Os    = 1,
	Bytes = 2,
}

impl From<PathKind> for StrandKind {
	fn from(value: PathKind) -> Self {
		match value {
			PathKind::Os => Self::Os,
			PathKind::Unix => Self::Bytes,
		}
	}
}

impl From<AuthKind> for StrandKind {
	fn from(value: AuthKind) -> Self {
		match value {
			AuthKind::Regular => Self::Os,
			AuthKind::Search => Self::Os,
			AuthKind::Mount => Self::Os,
			AuthKind::Hub => Self::Os,
			AuthKind::Scope => Self::Bytes,
			AuthKind::Sftp => Self::Bytes,
		}
	}
}
