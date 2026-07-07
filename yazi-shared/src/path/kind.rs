use serde::{Deserialize, Serialize};

use crate::scheme::SchemeKind;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
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
			SchemeKind::Sftp | SchemeKind::Rclone => Self::Unix,
		}
	}
}
