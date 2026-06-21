use std::fmt;

use dyn_clone::DynClone;
use yazi_macro::impl_data_any;

use crate::input::InputEvent;

pub trait InputCallback: Fn(InputEvent) + Send + Sync + DynClone + 'static {}

impl<T: Fn(InputEvent) + Send + Sync + Clone + 'static> InputCallback for T {}

impl Clone for Box<dyn InputCallback> {
	fn clone(&self) -> Self { dyn_clone::clone_box(&**self) }
}

impl fmt::Debug for dyn InputCallback {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("InputCallback").finish_non_exhaustive()
	}
}

impl_data_any!(Box<dyn InputCallback>);
