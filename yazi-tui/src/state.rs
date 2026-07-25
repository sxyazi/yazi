use crate::RatermOption;

#[derive(Clone, Copy)]
pub struct RatermState {
	pub mouse:        bool,
	pub title:        bool,
	pub cursor_shape: u8,
	pub cursor_blink: bool,
}

impl RatermState {
	pub(super) const fn default() -> Self {
		Self { mouse: false, title: false, cursor_shape: 0, cursor_blink: false }
	}

	pub(super) fn new(resp: &str, opt: &RatermOption) -> Self {
		let cursor_shape = resp
			.split_once("\x1bP1$r")
			.and_then(|(_, s)| s.bytes().next())
			.filter(|&b| matches!(b, b'0'..=b'6'))
			.map_or(u8::MAX, |b| b - b'0');

		let cursor_blink = resp.contains("\x1b[?12;1$y");

		Self { mouse: opt.mouse, title: false, cursor_shape, cursor_blink }
	}
}
