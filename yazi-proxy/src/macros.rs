#[macro_export]
macro_rules! deprecate {
	($content:expr) => {{
		static WARNED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
		if !WARNED.swap(true, std::sync::atomic::Ordering::Relaxed) {
			$crate::emit!(Call(
				yazi_shared::event::Cmd::new("app:deprecate").with("content", format!($tt, id))
			));
		}
	}};
}
