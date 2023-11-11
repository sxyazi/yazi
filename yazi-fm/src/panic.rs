use yazi_shared::Term;

pub(super) struct Panic;

impl Panic {
	pub(super) fn install() {
		better_panic::install();

		let hook = std::panic::take_hook();
		std::panic::set_hook(Box::new(move |info| {
			// TODO: remove this once https://github.com/rust-lang/rust/issues/108277 is fixed.
			if let Some(&s) = info.payload().downcast_ref::<&str>() {
				if s.contains("assertion failed: tv_nsec >= 0 && tv_nsec") {
					return;
				}
			}

			_ = Term::goodbye(|| {
				hook(info);
				true
			});
		}));
	}
}
