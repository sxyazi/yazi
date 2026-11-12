use std::ops::Range;

use mlua::{AnyUserData, UserData};

use super::FileSource;
use crate::file::File;

pub struct FilesInventory {
	/// Returns false if the userdata belongs to another type.
	pub visit:
		fn(&AnyUserData, Range<usize>, &mut dyn FnMut(&File) -> mlua::Result<()>) -> mlua::Result<bool>,
}

impl FilesInventory {
	pub const fn new<T: FileSource + UserData + 'static>() -> Self {
		Self {
			visit: |ud, range, visit| {
				let src = match ud.borrow::<T>() {
					Ok(src) => src,
					Err(mlua::Error::UserDataTypeMismatch) => return Ok(false),
					Err(e) => return Err(e),
				};

				for i in range.start..range.end.min(src.len()) {
					src.borrow(i, visit)?;
				}
				Ok(true)
			},
		}
	}
}

inventory::collect!(FilesInventory);
