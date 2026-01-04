use crate::scheme::SchemeKind;

pub enum PathKind {
	Os,
	Unix,
}

impl From<SchemeKind> for PathKind {
	fn from(value: SchemeKind) -> Self {
		match value {
			SchemeKind::Regular => Self::Os,
			SchemeKind::Search => Self::Os,
			SchemeKind::Archive => Self::Os,
			SchemeKind::Sftp => Self::Unix,
		}
	}
}
