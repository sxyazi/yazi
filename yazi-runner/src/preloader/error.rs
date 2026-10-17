use std::sync::Arc;

use strum::EnumIs;
use thiserror::Error;

#[derive(Clone, Debug, EnumIs, Error)]
pub enum PreloadError {
	#[error("Preload task cancelled")]
	Cancelled,
	#[error("Lua error during preload: {0}")]
	Lua(#[from] mlua::Error),
	#[error("Unexpected error during preload: {0}")]
	Unexpected(Arc<anyhow::Error>),
}

impl From<anyhow::Error> for PreloadError {
	fn from(e: anyhow::Error) -> Self { Self::Unexpected(e.into()) }
}
