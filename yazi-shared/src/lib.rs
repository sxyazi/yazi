#![allow(clippy::option_map_unit_fn)]

mod chars;
mod condition;
mod debounce;
mod env;
mod errors;
pub mod event;
pub mod fs;
mod layer;
mod natsort;
mod number;
mod os;
mod rand;
mod ro_cell;
pub mod shell;
mod terminal;
pub mod theme;
mod throttle;
mod time;
mod translit;
mod xdg;

pub use chars::*;
pub use condition::*;
pub use debounce::*;
pub use env::*;
pub use errors::*;
pub use layer::*;
pub use natsort::*;
pub use number::*;
#[cfg(unix)]
pub use os::*;
pub use rand::*;
pub use ro_cell::*;
pub use terminal::*;
pub use throttle::*;
pub use time::*;
pub use translit::*;
pub use xdg::*;

pub fn init() { event::Event::init(); }
