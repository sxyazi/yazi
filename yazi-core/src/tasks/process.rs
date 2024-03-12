use std::{borrow::Cow, collections::HashMap, ffi::OsString, mem};

use yazi_config::{open::Opener, OPEN};
use yazi_shared::fs::Url;

use super::Tasks;

impl Tasks {
	pub fn process_from_files(&self, hovered: Url, targets: Vec<(Url, String)>) {
		let mut openers = HashMap::new();
		for (url, mime) in targets {
			if let Some(opener) = OPEN.openers(&url, mime).and_then(|o| o.first().copied()) {
				openers.entry(opener).or_insert_with(|| vec![hovered.clone()]).push(url);
			}
		}
		for (opener, args) in openers {
			self.process_from_opener(
				Cow::Borrowed(opener),
				args.into_iter().map(|u| u.into_os_string()).collect(),
			);
		}
	}

	pub fn process_from_opener(&self, opener: Cow<'static, Opener>, mut args: Vec<OsString>) {
		if opener.spread {
			self.scheduler.process_open(opener, args, None);
			return;
		}
		if args.is_empty() {
			return;
		}
		if args.len() == 2 {
			self.scheduler.process_open(opener, args, None);
			return;
		}
		let hovered = mem::take(&mut args[0]);
		for target in args.into_iter().skip(1) {
			self.scheduler.process_open(opener.clone(), vec![hovered.clone(), target], None);
		}
	}
}
