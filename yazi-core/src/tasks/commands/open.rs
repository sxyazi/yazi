use yazi_config::open::Opener;
use yazi_shared::{emit, event::Exec, fs::Url, Layer};

use crate::tasks::Tasks;

pub struct Opt {
	targets: Vec<Url>,
	opener:  Opener,
}

impl TryFrom<Exec> for Opt {
	type Error = ();

	fn try_from(mut e: Exec) -> Result<Self, Self::Error> { e.take_data().ok_or(()) }
}

impl Tasks {
	pub fn _open(targets: Vec<Url>, opener: Opener) {
		emit!(Call(Exec::call("open", vec![]).with_data(Opt { targets, opener }), Layer::Tasks));
	}

	pub fn open(&mut self, opt: impl TryInto<Opt>) {
		if let Ok(opt) = opt.try_into() {
			self.file_open_with(&opt.opener, &opt.targets);
		}
	}
}
