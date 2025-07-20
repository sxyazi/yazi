#[macro_export]
macro_rules! render {
	() => {
		yazi_shared::event::NEED_RENDER.store(true, std::sync::atomic::Ordering::Relaxed);
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
