use std::process;

pub struct Actions;

impl Actions {
	pub(crate) fn act(args: &crate::Args) {
		if args.debug {
			println!("`yazi --debug` has been deprecated, use `ya env` instead.");
			process::exit(0);
		}

		if args.clear_cache {
			println!("`yazi --clear-cache` has been deprecated, use `ya cache clear` instead.");
			process::exit(0);
		}

		if args.version {
			println!("Yazi\n{}", yazi_version::version_full());
			process::exit(0);
		}
	}
}
