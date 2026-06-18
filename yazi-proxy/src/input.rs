use tokio::sync::mpsc;
use yazi_dds::Pubsub;
use yazi_macro::{emit, err, relay};
use yazi_shim::strum::IntoStr;
use yazi_widgets::input::{InputEvent, InputOpt};

pub struct InputProxy;

impl InputProxy {
	pub fn show(mut opt: InputOpt) -> mpsc::UnboundedReceiver<InputEvent> {
		let (tx, rx) = mpsc::unbounded_channel();
		opt = opt.with_cb(move |event| {
			err!(Pubsub::pub_after_input((&event).into_str(), event.value()));
			tx.send(event).ok();
		});

		emit!(Call(relay!(input:show).with_any("opt", opt)));
		rx
	}
}
