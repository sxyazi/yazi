use std::ffi::OsString;

use anyhow::anyhow;
use yazi_config::{open::Opener, BOOT};
use yazi_shared::{emit, event::Exec, Layer};

use crate::tasks::Tasks;

pub struct Opt {
	targets: Vec<(OsString, String)>,
	opener:  Option<Opener>,
}

impl TryFrom<&Exec> for Opt {
	type Error = anyhow::Error;

	fn try_from(e: &Exec) -> Result<Self, Self::Error> {
		e.take_data().ok_or_else(|| anyhow!("invalid data"))
	}
}

impl Tasks {
	pub fn _open(targets: Vec<(OsString, String)>, opener: Option<Opener>) {
		emit!(Call(Exec::call("open", vec![]).with_data(Opt { targets, opener }).vec(), Layer::Tasks));
	}

	pub fn open(&mut self, opt: impl TryInto<Opt>) -> bool {
		let Ok(opt) = opt.try_into() else {
			return false;
		};

		if let Some(p) = &BOOT.chooser_file {
			let paths = opt.targets.into_iter().map(|(p, _)| p.to_string_lossy().to_string()).collect::<Vec<_>>().join("\n");

			std::fs::write(p, paths.as_bytes()).ok();
			emit!(Quit(false));
			return false;
		}

		if let Some(opener) = opt.opener {
			self.file_open_with(&opener, &opt.targets.into_iter().map(|(f, _)| f).collect::<Vec<_>>());
		} else {
			self.file_open(&opt.targets);
		}
		false
	}
}
