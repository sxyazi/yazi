#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum CursorStyle {
	#[default]
	Default           = 0,
	BlinkingBlock     = 1,
	SteadyBlock       = 2,
	BlinkingUnderline = 3,
	SteadyUnderline   = 4,
	BlinkingBar       = 5,
	SteadyBar         = 6,
}
