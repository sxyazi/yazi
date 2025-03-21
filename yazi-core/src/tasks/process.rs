use std::{borrow::Cow, collections::HashMap, ffi::OsString, mem};

use yazi_config::{YAZI, opener::OpenerRule};
use yazi_proxy::options::ProcessExecOpt;
use yazi_shared::url::Url;

use super::Tasks;

impl Tasks {
	pub fn process_from_files(&self, cwd: Url, hovered: Url, targets: Vec<(Url, &str)>) {
		let mut openers = HashMap::new();
		for (url, mime) in targets {
			if let Some(opener) = YAZI.opener.first(YAZI.open.all(&url, mime)) {
				openers.entry(opener).or_insert_with(|| vec![hovered.clone()]).push(url);
			}
		}
		for (opener, args) in openers {
			self.process_from_opener(
				cwd.clone(),
				Cow::Borrowed(opener),
				args.into_iter().map(|u| u.into_path().into_os_string()).collect(),
			);
		}
	}

	pub fn process_from_opener(
		&self,
		cwd: Url,
		opener: Cow<'static, OpenerRule>,
		mut args: Vec<OsString>,
	) {
		if opener.spread {
			self.scheduler.process_open(ProcessExecOpt { cwd, opener, args, done: None });
			return;
		}
		if args.is_empty() {
			return;
		}
		if args.len() == 2 {
			self.scheduler.process_open(ProcessExecOpt { cwd, opener, args, done: None });
			return;
		}
		let hovered = mem::take(&mut args[0]);
		for target in args.into_iter().skip(1) {
			self.scheduler.process_open(ProcessExecOpt {
				cwd:    cwd.clone(),
				opener: opener.clone(),
				args:   vec![hovered.clone(), target],
				done:   None,
			});
		}
	}
}
