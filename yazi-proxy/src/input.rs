use tokio::sync::mpsc;
use yazi_config::popup::InputCfg;
use yazi_shared::{emit, event::Cmd, InputError, Layer};

pub struct InputOpt {
	pub cfg: InputCfg,
	pub tx:  mpsc::UnboundedSender<Result<String, InputError>>,
}

impl TryFrom<Cmd> for InputOpt {
	type Error = ();

	fn try_from(mut c: Cmd) -> Result<Self, Self::Error> { c.take_data().ok_or(()) }
}

pub struct InputProxy;

impl InputProxy {
	#[inline]
	pub fn show(cfg: InputCfg) -> mpsc::UnboundedReceiver<Result<String, InputError>> {
		let (tx, rx) = mpsc::unbounded_channel();
		emit!(Call(Cmd::new("show").with_data(InputOpt { cfg, tx }), Layer::Input));
		rx
	}

	#[inline]
	pub fn complete(word: &str, ticket: usize) {
		emit!(Call(Cmd::args("complete", vec![word.to_owned()]).with("ticket", ticket), Layer::Input));
	}
}
