#[macro_export]
macro_rules! render {
	() => {
		yazi_shared::event::NEED_RENDER.store(1, std::sync::atomic::Ordering::Relaxed);
	};
	($cond:expr) => {
		if $cond {
			render!();
		}
	};
	($left:expr, > $right:expr) => {{
		let val = $left;
		if val > $right {
			render!();
		}
		val
	}};
}

#[macro_export]
macro_rules! render_and {
	($cond:expr) => {
		if $cond {
			yazi_macro::render!();
			true
		} else {
			false
		}
	};
}

#[macro_export]
macro_rules! render_partial {
	() => {{
		_ = yazi_shared::event::NEED_RENDER.compare_exchange(
			0,
			2,
			std::sync::atomic::Ordering::Relaxed,
			std::sync::atomic::Ordering::Relaxed,
		);
	}};
}
