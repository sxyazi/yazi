use super::Style;

#[derive(Clone, Debug)]
pub struct Icon {
	pub text:  String,
	pub style: Style,
}

#[derive(Clone, Copy, Debug, Default)]
pub enum IconCache {
	#[default]
	Missing,
	Undefined,
	Icon(&'static Icon),
}
