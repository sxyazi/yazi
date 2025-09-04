#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum InputMode {
	Normal,
	#[default]
	Insert,
	Replace,
}

impl InputMode {
	#[inline]
	pub(super) fn delta(&self) -> usize { (*self != Self::Insert) as usize }
}
