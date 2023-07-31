pub struct Defer<F: FnOnce()>(Option<F>);

impl<F: FnOnce()> Defer<F> {
	pub fn new(f: F) -> Self { Defer(Some(f)) }
}

impl<F: FnOnce()> Drop for Defer<F> {
	fn drop(&mut self) {
		if let Some(f) = self.0.take() {
			f();
		}
	}
}
