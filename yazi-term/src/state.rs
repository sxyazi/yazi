use crate::TermOption;

#[derive(Clone, Copy)]
pub(super) struct TermState {
	pub(super) bg:           bool,
	pub(super) csi_u:        bool,
	pub(super) mouse:        bool,
	pub(super) title:        bool,
	pub(super) cursor_shape: u8,
	pub(super) cursor_blink: bool,
}

impl TermState {
	pub(super) const fn default() -> Self {
		Self {
			bg:           false,
			csi_u:        false,
			mouse:        false,
			title:        false,
			cursor_shape: 0,
			cursor_blink: false,
		}
	}

	pub(super) fn new(resp: &str, opt: &TermOption) -> Self {
		let csi_u = resp.contains("\x1b[?0u");

		let cursor_shape = resp
			.split_once("\x1bP1$r")
			.and_then(|(_, s)| s.bytes().next())
			.filter(|&b| matches!(b, b'0'..=b'6'))
			.map_or(u8::MAX, |b| b - b'0');

		let cursor_blink = resp.contains("\x1b[?12;1$y");

		Self {
			bg: !opt.bg.is_empty(),
			csi_u,
			mouse: opt.mouse,
			title: opt.title.is_some(),
			cursor_shape,
			cursor_blink,
		}
	}
}
