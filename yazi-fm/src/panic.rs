use crate::Term;

pub(super) struct Panic;

impl Panic {
	pub(super) fn install() {
		better_panic::install();

		let hook = std::panic::take_hook();
		std::panic::set_hook(Box::new(move |info| {
			Term::goodbye(|| {
				hook(info);
				1
			});
		}));
	}
}
