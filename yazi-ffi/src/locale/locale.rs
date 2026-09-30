#[derive(Clone, Copy)]
pub enum Width {
	Numeric,
	TwoDigit,
}

#[derive(Clone, Copy, Default)]
pub struct Locale {
	pub year:   Option<Width>,
	pub month:  Option<Width>,
	pub day:    Option<Width>,
	pub hour:   Option<Width>,
	pub minute: Option<Width>,
	pub second: Option<Width>,
	pub hour24: bool,
}
