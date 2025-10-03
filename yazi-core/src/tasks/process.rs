use std::{borrow::Cow, ffi::OsStr, mem};

use hashbrown::HashMap;
use yazi_config::{YAZI, opener::OpenerRule};
use yazi_parser::tasks::ProcessOpenOpt;
use yazi_shared::url::{UrlBuf, UrlCow};

use super::Tasks;

impl Tasks {
	pub fn process_with_selected(&self, cwd: UrlBuf, targets: Vec<(UrlCow<'static>, &str)>) {
		let mut openers = HashMap::new();
		for (url, mime) in targets {
			if let Some(opener) = YAZI.opener.first(YAZI.open.all(&url, mime)) {
				openers
					.entry(opener)
					.or_insert_with(|| vec![OsStr::new("").into()])
					.push(url.into_os_str2());
			}
		}
		for (opener, args) in openers {
			self.process_with_opener(cwd.clone(), Cow::Borrowed(opener), args);
		}
	}

	pub fn process_with_opener(
		&self,
		cwd: UrlBuf,
		opener: Cow<'static, OpenerRule>,
		mut args: Vec<Cow<'static, OsStr>>,
	) {
		if opener.spread {
			self.scheduler.process_open(ProcessOpenOpt { cwd, opener, args, done: None });
			return;
		}
		if args.is_empty() {
			return;
		}
		if args.len() == 2 {
			self.scheduler.process_open(ProcessOpenOpt { cwd, opener, args, done: None });
			return;
		}
		let hovered = mem::take(&mut args[0]);
		for target in args.into_iter().skip(1) {
			self.scheduler.process_open(ProcessOpenOpt {
				cwd:    cwd.clone(),
				opener: opener.clone(),
				args:   vec![hovered.clone(), target],
				done:   None,
			});
		}
	}
}
