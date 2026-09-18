use std::{hash::{Hash, Hasher}, ops::Deref};

use yazi_shared::url::Url;

use crate::{file::File, stat::StatSig};

#[derive(Clone, Copy, Debug)]
pub struct FileSig<'a>(pub &'a File);

impl Deref for FileSig<'_> {
	type Target = File;

	fn deref(&self) -> &Self::Target { self.0 }
}

impl Hash for FileSig<'_> {
	fn hash<H: Hasher>(&self, state: &mut H) {
		if let Some(backing) = self.extra.backing() {
			Url::regular(backing).hash(state);
		} else {
			self.url.hash(state);
		}

		StatSig(self.stat).hash(state);
		if self.is_link() {
			StatSig(self.lstat()).hash(state);
		}
	}
}
