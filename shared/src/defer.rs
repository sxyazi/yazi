pub struct Defer<F: FnOnce() -> T, T>(Option<F>);

impl<F: FnOnce() -> T, T> Defer<F, T> {
	pub fn new(f: F) -> Self { Defer(Some(f)) }
}

impl<F: FnOnce() -> T, T> Drop for Defer<F, T> {
	fn drop(&mut self) {
		if let Some(f) = self.0.take() {
			let _ = f();
		}
	}
}
