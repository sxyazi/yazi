use yazi_shared::{emit, event::Cmd, Layer};

pub struct CompletionProxy;

impl CompletionProxy {
	#[inline]
	pub fn close() {
		emit!(Call(Cmd::new("close"), Layer::Completion));
	}

	#[inline]
	pub fn trigger(word: &str, ticket: usize) {
		emit!(Call(
			Cmd::args("trigger", vec![word.to_owned()]).with("ticket", ticket),
			Layer::Completion
		));
	}
}
