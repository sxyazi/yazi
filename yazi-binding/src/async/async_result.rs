use compact_str::CompactString;
use yazi_macro::error;
use yazi_shim::fs::Error;

pub(super) struct AsyncResult<T> {
	pub(super) name:  CompactString,
	pub(super) value: Option<Result<T, Error>>,
}

impl<T> Drop for AsyncResult<T> {
	fn drop(&mut self) {
		if let Some(Err(error)) = &self.value {
			error!("Unhandled async block error in `{}`: {error}", self.name);
		}
	}
}
