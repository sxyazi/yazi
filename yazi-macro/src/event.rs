#[macro_export]
macro_rules! emit {
	(Quit($opt:expr)) => {
		yazi_shared::event::Event::Quit($opt).emit();
	};
	(Call($cmd:expr)) => {
		yazi_shared::event::Event::Call(yazi_shared::event::CmdCow::from($cmd)).emit();
	};
	(Seq($cmds:expr)) => {
		yazi_shared::event::Event::Seq($cmds).emit();
	};
	($event:ident) => {
		yazi_shared::event::Event::$event.emit();
	};
}

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
