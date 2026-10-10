yazi_macro::mod_flat!(behavior cache ctx entries file filter finder folder input input_alt lives mode mut_cell preference preview ptr selected tab tabs task tasks which yanked);

pub(crate) fn init() { unsafe { TABS.get().write(std::mem::MaybeUninit::new(<_>::default())) }; }
