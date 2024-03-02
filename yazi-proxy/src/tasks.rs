use yazi_config::open::Opener;
use yazi_shared::{emit, event::Cmd, fs::Url, Layer};

pub struct TasksProxy;

pub struct OpenWithOpt {
	pub targets: Vec<Url>,
	pub opener:  Opener,
}

impl TryFrom<Cmd> for OpenWithOpt {
	type Error = ();

	fn try_from(mut c: Cmd) -> Result<Self, Self::Error> { c.take_data().ok_or(()) }
}

impl TasksProxy {
	#[inline]
	pub fn open_with(targets: Vec<Url>, opener: Opener) {
		emit!(Call(Cmd::new("open_with").with_data(OpenWithOpt { targets, opener }), Layer::Tasks));
	}
}
