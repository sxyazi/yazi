use std::process;

pub struct Actions;

impl Actions {
	// Actions that require no terminal/config initialization, handled before it.
	pub(crate) fn act_early(args: &crate::Args) {
		if args.version {
			println!("Yazi {}", Self::version());
			process::exit(0);
		}
	}

	pub(crate) fn act(args: &crate::Args) {
		if args.debug {
			println!("{}", Self::debug().unwrap());
			process::exit(0);
		}

		if args.clear_cache {
			Self::clear_cache();
			process::exit(0);
		}
	}
}
