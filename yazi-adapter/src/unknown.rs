use crate::Adapter;

#[derive(Debug, Clone, Copy)]
pub struct Unknown {
	pub kgp:   bool,
	pub sixel: bool,
}

impl Unknown {
	pub(super) const fn default() -> Self { Self { kgp: false, sixel: false } }

	pub(super) fn adapters(self) -> &'static [Adapter] {
		use Adapter as A;

		match (self.kgp, self.sixel) {
			(true, true) => &[A::Kgp, A::Sixel],
			(true, false) => &[A::Kgp],
			(false, true) => &[A::Sixel],
			(false, false) => &[],
		}
	}
}
