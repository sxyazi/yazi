#[macro_export]
macro_rules! deprecate {
	($content:expr) => {{
		static WARNED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
		if !WARNED.swap(true, std::sync::atomic::Ordering::Relaxed) {
			$crate::AppProxy::notify($crate::options::NotifyOpt {
				title:   "Deprecated API".to_owned(),
				content: $content.to_owned(),
				level:   $crate::options::NotifyLevel::Warn,
				timeout: std::time::Duration::from_secs(20),
			});
		}
	}};
}
