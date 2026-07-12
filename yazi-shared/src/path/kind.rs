use serde::{Deserialize, Serialize};

use crate::auth::AuthKind;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PathKind {
	Os,
	Unix,
}

impl From<AuthKind> for PathKind {
	fn from(value: AuthKind) -> Self {
		match value {
			AuthKind::Regular => Self::Os,
			AuthKind::Search => Self::Os,
			AuthKind::Mount => Self::Os,
			AuthKind::Scope => Self::Unix,
			AuthKind::Sftp => Self::Unix,
		}
	}
}
