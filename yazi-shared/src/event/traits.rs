use anyhow::Result;

use super::ActionCow;

pub trait FromAction<C, A = ActionCow>: Sized {
	fn from_action(action: A, cx: &C) -> Result<Self>;
}

impl<C, A, T> FromAction<C, A> for T
where
	T: TryFrom<A>,
	T::Error: Into<anyhow::Error>,
{
	fn from_action(action: A, _: &C) -> Result<Self> { action.try_into().map_err(Into::into) }
}
