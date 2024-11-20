#![allow(
	clippy::if_same_then_else,
	clippy::len_without_is_empty,
	clippy::module_inception,
	clippy::option_map_unit_fn,
	clippy::unit_arg
)]

yazi_macro::mod_pub!(completion confirm help input manager notify pick spot tab tasks which);

pub fn init() {
	manager::WATCHED.with(<_>::default);
	manager::LINKED.with(<_>::default);
}
