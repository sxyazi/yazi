yazi_macro::mod_flat!(brand deinit dimension emulator mux probe);

pub fn init() { EMULATOR.init(Emulator::from_env()); }

pub fn setup() -> anyhow::Result<Deinit> {
	let deinit = Deinit;
	EMULATOR.start()?;

	Ok(deinit)
}
