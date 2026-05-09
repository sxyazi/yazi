#[macro_export]
macro_rules! input {
	($cx:ident, $cfg:expr) => {{
		let (tx, rx) = ::tokio::sync::mpsc::unbounded_channel();
		match $crate::act!(input:show, $cx, yazi_widgets::input::InputOpt { cfg: $cfg, tx }) {
			Ok(_) => Ok(rx),
			Err(e) => Err(e)
		}
	}};
}
