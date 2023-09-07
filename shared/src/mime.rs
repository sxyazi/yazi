pub const MIME_DIR: &str = "inode/directory";

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MimeKind {
	Empty,

	Archive,

	Image,
	Video,

	JSON,
	PDF,
	Text,

	Others,
}

impl MimeKind {
	pub fn new(s: &str) -> Self {
		if s.starts_with("text/") || s.ends_with("/xml") || s.ends_with("/javascript") {
			Self::Text
		} else if s.starts_with("image/") {
			Self::Image
		} else if s.starts_with("video/") {
			Self::Video
		} else if s == "inode/x-empty" {
			Self::Empty
		} else if s == "application/json" {
			Self::JSON
		} else if s == "application/pdf" {
			Self::PDF
		} else if s == "application/zip"
			|| s == "application/gzip"
			|| s == "application/x-tar"
			|| s == "application/x-bzip"
			|| s == "application/x-bzip2"
			|| s == "application/x-7z-compressed"
			|| s == "application/x-rar"
		{
			Self::Archive
		} else {
			Self::Others
		}
	}

	pub fn valid(s: &str) -> bool {
		if s == "inode/x-empty" {
			return true;
		}

		let parts = s.split('/').collect::<Vec<_>>();
		if parts.len() != 2 {
			return false;
		}

		#[rustfmt::skip]
		let b = matches!(parts[0], "application" | "audio" | "example" | "font" | "image" | "message" | "model" | "multipart" | "text" | "video");
		b && !parts[1].is_empty()
	}

	pub fn show_as_image(&self) -> bool {
		matches!(self, MimeKind::Image | MimeKind::Video | MimeKind::PDF)
	}
}
