#![allow(
	clippy::if_same_then_else,
	clippy::len_without_is_empty,
	clippy::module_inception,
	clippy::option_map_unit_fn,
	clippy::unit_arg
)]

yazi_macro::mod_flat!(scrollable);

yazi_macro::mod_pub!(cmp confirm help input mgr notify pick spot tab tasks which);

pub fn init() {
	mgr::WATCHED.with(<_>::default);
	mgr::LINKED.with(<_>::default);
}
