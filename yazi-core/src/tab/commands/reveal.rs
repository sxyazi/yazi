use yazi_config::keymap::Exec;
use yazi_shared::Url;

use crate::{emit, files::{File, FilesOp}, tab::Tab};

pub struct Opt<'a> {
	target: &'a str,
}

impl<'a> From<&'a Exec> for Opt<'a> {
	fn from(e: &'a Exec) -> Self { Self { target: e.args.first().map(|s| s.as_str()).unwrap_or("") } }
}

impl Tab {
	pub fn reveal<'a>(&mut self, opt: impl Into<Opt<'a>>) -> bool {
		let opt = opt.into() as Opt;

		let target = Url::from(opt.target);
		let Some(parent) = target.parent_url() else {
			return false;
		};

		let b = self.cd(parent.clone());
		emit!(Files(FilesOp::Creating(parent.clone(), File::from_dummy(target.clone()).into_map())));
		emit!(Hover(target));
		b
	}
}
