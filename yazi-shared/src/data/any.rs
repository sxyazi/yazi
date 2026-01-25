use std::{any::Any, fmt};

use dyn_clone::DynClone;

pub trait DataAny: Any + Send + Sync + DynClone {
	fn as_any(&self) -> &dyn Any;

	fn into_any(self: Box<Self>) -> Box<dyn Any>;
}

impl<T> DataAny for T
where
	T: Any + Send + Sync + DynClone,
{
	fn as_any(&self) -> &dyn Any { self }

	fn into_any(self: Box<Self>) -> Box<dyn Any> { self }
}

impl Clone for Box<dyn DataAny> {
	fn clone(&self) -> Self { dyn_clone::clone_box(&**self) }
}

impl fmt::Debug for dyn DataAny {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("DataAny").finish_non_exhaustive()
	}
}
