use std::{ffi::OsStr, path::Path};

pub trait OsStrExt2 {
	fn is_filename(&self) -> bool;
}

impl OsStrExt2 for OsStr {
	fn is_filename(&self) -> bool { Path::new(self).file_name() == Some(self) }
}
