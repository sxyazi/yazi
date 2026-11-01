yazi_macro::mod_flat!(args boot);

pub fn init() {
	use clap::Parser;

	BOOT.init(Args::parse().into());
}

pub fn init_default() { BOOT.with(Default::default); }
