use std::{ffi::OsString, path::MAIN_SEPARATOR_STR};

#[derive(Debug, Clone)]
pub struct CmpItem {
	pub name:   OsString,
	pub is_dir: bool,
}

impl CmpItem {
	pub fn completable(&self) -> String {
		format!("{}{}", self.name.to_string_lossy(), if self.is_dir { MAIN_SEPARATOR_STR } else { "" })
	}
}
