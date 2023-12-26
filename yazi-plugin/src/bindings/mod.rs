#![allow(clippy::module_inception)]

mod bindings;
mod cha;
mod file;
mod range;
mod url;
mod window;

pub use bindings::*;
pub use cha::*;
pub use file::*;
pub use range::*;
pub use url::*;
pub use window::*;

pub trait Cast<T> {
	fn cast(lua: &mlua::Lua, data: T) -> mlua::Result<mlua::AnyUserData>;
}
