use crate::path::PathKind;

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
