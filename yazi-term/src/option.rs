use yazi_config::{THEME, YAZI};

pub(super) struct TermOption {
	pub bg:    String,
	pub mouse: bool,
	pub title: Option<String>,
}

impl Default for TermOption {
	fn default() -> Self {
		Self {
			bg:    THEME.app.bg_color(),
			mouse: !YAZI.mgr.mouse_events.get().is_empty(),
			title: YAZI.mgr.title(),
		}
	}
}
