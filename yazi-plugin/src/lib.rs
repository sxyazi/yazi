#![allow(clippy::unit_arg)]

pub mod bindings;
mod cast;
mod config;
pub mod elements;
pub mod external;
pub mod fs;
pub mod isolate;
pub mod loader;
mod lua;
mod opt;
pub mod process;
pub mod pubsub;
mod runtime;
pub mod url;
pub mod utils;

pub use cast::*;
pub use config::*;
pub use lua::*;
pub use opt::*;
pub use runtime::*;

pub fn init() {
	crate::loader::init();
	crate::init_lua();
}
