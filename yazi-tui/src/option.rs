use yazi_config::{THEME, YAZI};

pub(super) struct RatermOption {
	pub bg:    String,
	pub mouse: bool,
}

impl Default for RatermOption {
	fn default() -> Self {
		Self { bg: THEME.app.bg_color(), mouse: !YAZI.mgr.mouse_events.get().is_empty() }
	}
}
