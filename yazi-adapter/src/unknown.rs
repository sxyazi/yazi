use crate::Adapter;

#[derive(Clone, Copy, Debug, Default)]
pub struct Unknown {
	pub kgp:   bool,
	pub sixel: bool,
}

impl Unknown {
	pub(super) fn adapters(self) -> &'static [Adapter] {
		use Adapter as A;

		match (self.kgp, self.sixel) {
			(true, true) => &[A::Sixel, A::KgpOld],
			(true, false) => &[A::KgpOld],
			(false, true) => &[A::Sixel],
			(false, false) => &[],
		}
	}
}
