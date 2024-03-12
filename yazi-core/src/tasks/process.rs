use std::{collections::HashMap, ffi::OsStr};

use yazi_config::{open::Opener, OPEN};
use yazi_shared::fs::Url;

use super::Tasks;

impl Tasks {
	pub fn process_from_files(&self, hovered: &Url, targets: &[(Url, String)]) {
		let mut openers = HashMap::new();
		for (url, mime) in targets {
			if let Some(opener) = OPEN.openers(url, mime).and_then(|o| o.first().copied()) {
				openers.entry(opener).or_insert_with(|| vec![hovered]).push(url);
			}
		}
		for (opener, args) in openers {
			self.process_from_opener(opener, &args);
		}
	}

	pub fn process_from_opener(&self, opener: &Opener, args: &[impl AsRef<OsStr>]) {
		if opener.spread {
			self.scheduler.process_open(opener, args);
			return;
		}
		for target in args.iter().skip(1) {
			self.scheduler.process_open(opener, &[&args[0], target]);
		}
	}
}
