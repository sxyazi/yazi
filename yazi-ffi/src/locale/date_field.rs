use strum::EnumIs;

#[derive(Clone, Copy, EnumIs)]
pub(super) enum DateField {
	Year,
	Month,
	Day,
}
