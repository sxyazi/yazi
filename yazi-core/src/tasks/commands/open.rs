use yazi_config::open::Opener;
use yazi_shared::{emit, event::Cmd, fs::Url, Layer};

use crate::tasks::Tasks;

pub struct Opt {
	targets: Vec<Url>,
	opener:  Opener,
}

impl TryFrom<Cmd> for Opt {
	type Error = ();

	fn try_from(mut c: Cmd) -> Result<Self, Self::Error> { c.take_data().ok_or(()) }
}

impl Tasks {
	pub fn _open(targets: Vec<Url>, opener: Opener) {
		emit!(Call(Cmd::new("open").with_data(Opt { targets, opener }), Layer::Tasks));
	}

	pub fn open(&mut self, opt: impl TryInto<Opt>) {
		if let Ok(opt) = opt.try_into() {
			self.file_open_with(&opt.opener, &opt.targets);
		}
	}
}
