yazi_macro::mod_flat!(args build common dest help install);

fn main() -> anyhow::Result<()> {
	match Args::parse()? {
		Args::Build(target) => Build::new(target).run(),
		Args::Dest(target) => Dest::new(target).run(),
		Args::Install(bin_dir) => Install::new(bin_dir).run(),
		Args::Help => Help::run(),
	}
}
