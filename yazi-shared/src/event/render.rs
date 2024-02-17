use std::sync::atomic::AtomicBool;

pub static NEED_RENDER: AtomicBool = AtomicBool::new(false);

#[macro_export]
macro_rules! render {
	() => {
		$crate::event::NEED_RENDER.store(true, std::sync::atomic::Ordering::Relaxed);
	};
	($cond:expr) => {
		if $cond {
			render!();
		}
	};
}

#[macro_export]
macro_rules! render_and {
	($cond:expr) => {
		if $cond {
			render!();
			true
		} else {
			false
		}
	};
}
