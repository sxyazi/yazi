use std::sync::Arc;

use mlua::{IntoLua, Lua, Value};
use thiserror::Error;

#[derive(Clone, Debug, Error)]
pub enum PeekError {
	#[error("Sync previewer")]
	ShouldSync,
	#[error("Peek task cancelled")]
	Cancelled,
	#[error("Peek exceeded upper bound of {0}")]
	Exceeded(usize),
	#[error("Lua error during peek: {0}")]
	Lua(#[from] mlua::Error),
	#[error("Unexpected error during peek: {0}")]
	Unexpected(Arc<anyhow::Error>),
}

impl From<anyhow::Error> for PeekError {
	fn from(e: anyhow::Error) -> Self { Self::Unexpected(e.into()) }
}

impl From<tokio::task::JoinError> for PeekError {
	fn from(e: tokio::task::JoinError) -> Self { Self::Unexpected(Arc::new(e.into())) }
}

impl IntoLua for PeekError {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> { self.to_string().into_lua(lua) }
}
