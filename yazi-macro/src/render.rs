#[macro_export]
macro_rules! render {
	() => {
		_ = yazi_shared::event::NEED_RENDER.fetch_max(
			2, // normal
			std::sync::atomic::Ordering::Relaxed,
		)
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
		_ = yazi_shared::event::NEED_RENDER.fetch_max(
			1, // partial
			std::sync::atomic::Ordering::Relaxed,
		);
	}};
}

#[macro_export]
macro_rules! render_force {
	() => {
		_ = yazi_shared::event::NEED_RENDER.fetch_max(
			3, // force
			std::sync::atomic::Ordering::Relaxed,
		)
	};
}
