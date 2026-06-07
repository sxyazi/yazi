use yazi_config::YAZI;

pub(super) struct RatermOption {
	pub mouse: bool,
}

impl Default for RatermOption {
	fn default() -> Self { Self { mouse: !YAZI.mgr.mouse_events.get().is_empty() } }
}
