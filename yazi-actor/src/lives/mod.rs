yazi_macro::mod_flat!(behavior core file files filter finder folder lives mode mut_cell preference preview ptr selected tab tabs task tasks which yanked);

pub(super) fn init() {
	unsafe { FILE_CACHE.get().write(std::mem::MaybeUninit::new(<_>::default())) };
}
