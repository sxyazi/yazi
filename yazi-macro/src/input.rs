#[macro_export]
macro_rules! input {
	($cx:ident, $opt:expr) => {{
		use yazi_dds::Pubsub;
		use yazi_shim::strum::IntoStr;
		use yazi_widgets::input::{InputCallback, InputEvent, InputOpt};

		let (tx, rx) = ::tokio::sync::mpsc::unbounded_channel();
		let opt = $opt.with_cb(move |event| {
			$crate::err!(Pubsub::pub_after_input((&event).into_str(), event.value()));
			tx.send(event).ok();
		});

		match $crate::act!(input:show, $cx, opt) {
			Ok(_) => Ok(rx),
			Err(e) => Err(e)
		}
	}};
}
