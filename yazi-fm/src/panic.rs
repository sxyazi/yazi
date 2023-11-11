use yazi_shared::Term;

pub(super) struct Panic;

impl Panic {
	pub(super) fn install() {
		better_panic::install();

		let hook = std::panic::take_hook();
		std::panic::set_hook(Box::new(move |info| {
			_ = Term::goodbye(|| {
				hook(info);
				true
			});
		}));
	}
}
