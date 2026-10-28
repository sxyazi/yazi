use std::ops::{Deref, DerefMut};

use yazi_config::YAZI;
use yazi_fs::file::File;
use yazi_macro::render;
use yazi_runner::{RUNNER, spot::SpotJob};
use yazi_shared::{pool::Symbol, scope::Scope};

use super::Spot;
use crate::tab::Tab;

pub struct SpotMut<'a> {
	inner:  &'a mut Spot,
	parent: &'a Scope,
}

impl Deref for SpotMut<'_> {
	type Target = Spot;

	fn deref(&self) -> &Self::Target { self.inner }
}

impl DerefMut for SpotMut<'_> {
	fn deref_mut(&mut self) -> &mut Self::Target { self.inner }
}

impl<'a> From<&'a mut Tab> for SpotMut<'a> {
	fn from(tab: &'a mut Tab) -> Self { Self { inner: &mut tab.spot, parent: &tab.scope } }
}

impl SpotMut<'_> {
	fn renew(&mut self) {
		self.scope.cancel();
		self.scope = self.parent.child();
	}

	pub fn reset(&mut self) {
		self.renew();
		render!(self.lock.take().is_some());
	}

	pub fn go(mut self, file: File, mime: Symbol<str>, force: bool) {
		if mime.is_empty() {
			return; // Wait till mimetype is resolved to avoid flickering
		} else if !force && self.same_lock(&file, &mime) {
			return;
		}

		let Some(spotter) = YAZI.plugin.spotters.matches(&file, &mime) else {
			return self.reset();
		};

		self.renew();
		RUNNER.spot(SpotJob { spotter, file, mime, skip: self.skip, scope: self.scope.clone() });
	}
}
