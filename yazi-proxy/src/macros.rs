#[macro_export]
macro_rules! deprecate {
	($content:expr) => {{
		static WARNED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
		if !WARNED.swap(true, std::sync::atomic::Ordering::Relaxed) {
			yazi_macro::emit!(Call(
				yazi_shared::event::Action::new_relay("app:deprecate").with("content", $content)
			));
		}
	}};
}
