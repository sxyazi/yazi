#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum InputMode {
	Normal,
	#[default]
	Insert,
}

impl InputMode {
	#[inline]
	pub(super) fn delta(&self) -> usize { (*self != InputMode::Insert) as usize }
}
