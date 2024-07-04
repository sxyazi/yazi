#![allow(clippy::unit_arg)]

pub mod bindings;
mod cast;
pub mod cha;
mod clipboard;
mod config;
pub mod elements;
pub mod external;
pub mod file;
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
pub use clipboard::*;
pub use config::*;
pub use lua::*;
pub use opt::*;
pub use runtime::*;

pub fn init() -> anyhow::Result<()> {
	CLIPBOARD.with(Default::default);

	crate::loader::init();
	crate::init_lua()?;
	Ok(())
}
