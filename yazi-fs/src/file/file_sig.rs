use std::{hash::{Hash, Hasher}, ops::Deref};

use yazi_shared::url::Url;

use crate::{cha::ChaSig, file::File};

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

		ChaSig(self.cha).hash(state);
	}
}
