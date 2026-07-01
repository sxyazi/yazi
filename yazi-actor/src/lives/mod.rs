yazi_macro::mod_flat!(behavior core entries file filter finder folder input input_alt lives mode mut_cell preference preview ptr selected tab tabs task tasks which yanked);

pub(super) fn init() {
	unsafe { FILE_CACHE.get().write(std::mem::MaybeUninit::new(<_>::default())) };
}
