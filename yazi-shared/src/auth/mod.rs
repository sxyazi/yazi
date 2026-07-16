yazi_macro::mod_flat!(auth domain encode inventory kind scheme);

pub(super) fn init() { DEFAULT_ARC.with(Default::default); }
