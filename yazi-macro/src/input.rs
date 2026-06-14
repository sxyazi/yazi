#[macro_export]
macro_rules! input {
	($cx:ident, $cfg:expr) => {{
		use yazi_dds::Pubsub;
		use yazi_shim::strum::IntoStr;
		use yazi_widgets::input::{InputCallback, InputEvent, InputOpt};

		let (tx, rx) = ::tokio::sync::mpsc::unbounded_channel();
		let cb: Box<dyn InputCallback> = Box::new(move |event| {
			$crate::err!(Pubsub::pub_after_input((&event).into_str(), event.value()));
			tx.send(event).ok();
		});

		match $crate::act!(input:show, $cx, InputOpt { cfg: $cfg, cb: Some(cb) }) {
			Ok(_) => Ok(rx),
			Err(e) => Err(e)
		}
	}};
}
