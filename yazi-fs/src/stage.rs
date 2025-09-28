use serde::{Deserialize, Serialize};

use crate::error::Error;

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum FolderStage {
	#[default]
	Loading,
	Loaded,
	Failed(Error),
}

impl FolderStage {
	pub fn is_loading(&self) -> bool { *self == Self::Loading }
}
