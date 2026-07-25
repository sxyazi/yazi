use std::path::{Component, Path};

pub trait PathExt {
	fn has_parent_component(&self) -> bool;
}

impl PathExt for Path {
	fn has_parent_component(&self) -> bool {
		self.components().any(|c| matches!(c, Component::ParentDir))
	}
}
