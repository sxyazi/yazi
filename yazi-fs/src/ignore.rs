use std::path::Path;

use yazi_shared::url::AsUrl;

// yazi-fs cannot depend on yazi-config, so the exclude rules come in as a closure
pub struct IgnoreFilter(Box<dyn Fn(&Path) -> bool + Send + Sync>);

impl IgnoreFilter {
	pub fn new(f: impl Fn(&Path) -> bool + Send + Sync + 'static) -> Self { Self(Box::new(f)) }

	pub fn matches_url(&self, url: impl AsUrl) -> bool {
		let loc = url.as_url().loc();
		match loc.as_os() {
			Ok(path) => (self.0)(path),
			Err(_) => (self.0)(Path::new(&*loc.to_string_lossy())),
		}
	}
}
