use yazi_config::open::Opener;
use yazi_shared::{emit, event::Cmd, fs::Url, Layer};

use crate::options::OpenWithOpt;

pub struct TasksProxy;

impl TasksProxy {
	#[inline]
	pub fn open_with(targets: Vec<Url>, opener: Opener) {
		emit!(Call(Cmd::new("open_with").with_data(OpenWithOpt { targets, opener }), Layer::Tasks));
	}
}
