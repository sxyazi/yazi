#![allow(
	clippy::if_same_then_else,
	clippy::len_without_is_empty,
	clippy::module_inception,
	clippy::option_map_unit_fn,
	clippy::unit_arg
)]

pub mod completion;
pub mod folder;
pub mod help;
pub mod input;
pub mod manager;
pub mod notify;
pub mod select;
mod step;
pub mod tab;
pub mod tasks;
pub mod which;

pub use step::*;

pub fn init() {
	manager::WATCHED.with(<_>::default);
	manager::LINKED.with(<_>::default);
}
