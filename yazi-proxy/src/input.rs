use tokio::sync::mpsc;
use yazi_config::popup::InputCfg;
use yazi_shared::{emit, event::Cmd, InputError, Layer};

pub struct InputProxy;

impl InputProxy {
	#[inline]
	pub fn show(cfg: InputCfg) -> mpsc::UnboundedReceiver<Result<String, InputError>> {
		let (tx, rx) = mpsc::unbounded_channel();
		emit!(Call(Cmd::new("show").with_any("tx", tx).with_any("cfg", cfg), Layer::Input));
		rx
	}

	#[inline]
	pub fn complete(word: &str, ticket: usize) {
		emit!(Call(Cmd::args("complete", &[word]).with("ticket", ticket), Layer::Input));
	}
}
