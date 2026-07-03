use std::ops::{Deref, DerefMut};

use yazi_shared::url::UrlBuf;

use crate::file::File;

#[derive(Clone, Debug, Default)]
pub struct Files(pub Vec<File>);

impl Deref for Files {
	type Target = Vec<File>;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl DerefMut for Files {
	fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

impl From<Vec<File>> for Files {
	fn from(value: Vec<File>) -> Self { Self(value) }
}

impl From<Files> for Vec<File> {
	fn from(value: Files) -> Self { value.0 }
}

impl From<Files> for Vec<UrlBuf> {
	fn from(value: Files) -> Self { value.0.into_iter().map(|f| f.url).collect() }
}
