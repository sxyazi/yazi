use yazi_fs::File;

use crate::Pattern;

pub trait Selectable {
	fn url_pat(&self) -> Option<&Pattern>;

	fn mime_pat(&self) -> Option<&Pattern>;

	#[inline]
	fn matches(&self, file: &File, mime: &str) -> bool { self.match_with(Some(file), Some(mime)) }

	fn match_with(&self, file: Option<&File>, mime: Option<&str>) -> bool {
		match (self.url_pat(), self.mime_pat(), file, mime) {
			(_, Some(mp), _, Some(m)) if !mp.match_mime(m) => false,
			(Some(up), _, Some(f), _) if !up.match_url(&f.url, f.is_dir()) => false,
			(_, Some(_), _, None) | (Some(_), _, None, _) => false,
			_ => true,
		}
	}
}
