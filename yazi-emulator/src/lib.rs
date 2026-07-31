yazi_macro::mod_flat!(brand dimension emulator mux probe);

pub fn init() { EMULATOR.init(arc_swap::ArcSwap::from_pointee(Emulator::from_env())); }
